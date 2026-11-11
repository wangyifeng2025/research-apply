use axum::Router;
use axum::body::Body;
use axum::http::{Method, Request, StatusCode};
use entity::sea_orm_active_enums::{ApplicationStatus, ProjectClass};
use entity::{application_cycles, applications, departments, users};
use http_body_util::BodyExt;
use research_apply_server::auth::encode_session;
use research_apply_server::config::Config;
use research_apply_server::routes;
use research_apply_server::state::AppState;
use research_apply_server::support::time::now;
use sea_orm::{ActiveModelTrait, ActiveValue::Set, ColumnTrait, EntityTrait, QueryFilter};
use serde_json::{Value, json};
use tower::ServiceExt;
use uuid::Uuid;

#[tokio::test]
#[ignore = "需要本地 PostgreSQL 和演示账号"]
async fn department_review_checks_scope_and_rejects_a_second_decision() {
    dotenvy::dotenv().ok();
    let config = Config::from_env().unwrap_or_else(|error| panic!("{error}"));
    let state = AppState::connect(&config)
        .await
        .unwrap_or_else(|error| panic!("{error}"));
    let app = routes::router(state.clone(), &config).unwrap_or_else(|error| panic!("{error}"));
    let reviewer = user_by_name(&state, "liwenbo").await;
    let teacher = user_by_name(&state, "zhaomingyuan").await;
    let office = token(&state, "wanghaining").await;
    let admin = token(&state, "zhoujin").await;
    let reviewer_token = encode_session(reviewer.id, &state.config.jwt_secret)
        .unwrap_or_else(|error| panic!("{error}"));
    let teacher_token = encode_session(teacher.id, &state.config.jwt_secret)
        .unwrap_or_else(|error| panic!("{error}"));
    let own_department = reviewer
        .department_id
        .unwrap_or_else(|| panic!("系审核账号没有部门"));
    let other_department = other_department(&state, own_department).await;
    let cycle = application_cycles::Entity::find()
        .one(&state.db)
        .await
        .unwrap_or_else(|error| panic!("{error}"))
        .unwrap_or_else(|| panic!("缺少申报批次"));
    let own_id = insert_submitted(
        &state,
        cycle.id,
        teacher.id,
        own_department,
        "系审核接口测试",
    )
    .await;
    let other_id = insert_submitted(
        &state,
        cycle.id,
        teacher.id,
        other_department,
        "外部门接口测试",
    )
    .await;

    let result = exercise(
        &app,
        &reviewer_token,
        &teacher_token,
        &office,
        &admin,
        own_id,
        other_id,
    )
    .await;
    let _ = applications::Entity::delete_by_id(own_id)
        .exec(&state.db)
        .await;
    let _ = applications::Entity::delete_by_id(other_id)
        .exec(&state.db)
        .await;
    if let Err(error) = result {
        panic!("{error}");
    }
}

async fn exercise(
    app: &Router,
    reviewer: &str,
    teacher: &str,
    office: &str,
    admin: &str,
    own_id: Uuid,
    other_id: Uuid,
) -> Result<(), String> {
    expect_status(
        app,
        Method::GET,
        "/api/department-reviews?scope=pending",
        Some(teacher),
        None,
        StatusCode::FORBIDDEN,
    )
    .await?;
    expect_status(
        app,
        Method::GET,
        "/api/department-reviews?scope=pending",
        Some(office),
        None,
        StatusCode::FORBIDDEN,
    )
    .await?;

    let pending = expect_status(
        app,
        Method::GET,
        "/api/department-reviews?scope=pending",
        Some(reviewer),
        None,
        StatusCode::OK,
    )
    .await?;
    if !contains_id(&pending, own_id) || contains_id(&pending, other_id) {
        return Err(format!("待审范围不正确：{pending}"));
    }
    let admin_pending = expect_status(
        app,
        Method::GET,
        "/api/department-reviews?scope=pending",
        Some(admin),
        None,
        StatusCode::OK,
    )
    .await?;
    if !contains_id(&admin_pending, own_id) || !contains_id(&admin_pending, other_id) {
        return Err("管理员应能看到全院待审申报".to_owned());
    }

    expect_status(
        app,
        Method::GET,
        &format!("/api/department-reviews/{own_id}"),
        Some(reviewer),
        None,
        StatusCode::OK,
    )
    .await?;
    expect_status(
        app,
        Method::POST,
        &format!("/api/department-reviews/{own_id}"),
        Some(reviewer),
        Some(decision("approved", "", false)),
        StatusCode::UNPROCESSABLE_ENTITY,
    )
    .await?;
    expect_status(
        app,
        Method::POST,
        &format!("/api/department-reviews/{other_id}"),
        Some(reviewer),
        Some(decision("approved", "同意", true)),
        StatusCode::FORBIDDEN,
    )
    .await?;

    let reviewed = expect_status(
        app,
        Method::POST,
        &format!("/api/department-reviews/{own_id}"),
        Some(reviewer),
        Some(decision("approved", "同意推荐上报科研处", true)),
        StatusCode::OK,
    )
    .await?;
    if reviewed["decision"] != "approved" || reviewed["status"] != "dept_approved" {
        return Err(format!("审核结果不正确：{reviewed}"));
    }
    expect_status(
        app,
        Method::POST,
        &format!("/api/department-reviews/{own_id}"),
        Some(reviewer),
        Some(decision("returned", "再改一次", true)),
        StatusCode::CONFLICT,
    )
    .await?;
    expect_status(
        app,
        Method::POST,
        &format!("/api/applications/{own_id}/withdraw"),
        Some(teacher),
        None,
        StatusCode::CONFLICT,
    )
    .await?;

    let pending_after = expect_status(
        app,
        Method::GET,
        "/api/department-reviews?scope=pending",
        Some(reviewer),
        None,
        StatusCode::OK,
    )
    .await?;
    if contains_id(&pending_after, own_id) {
        return Err("已审申报仍留在待审列表".to_owned());
    }
    let mine = expect_status(
        app,
        Method::GET,
        "/api/department-reviews?scope=reviewed",
        Some(reviewer),
        None,
        StatusCode::OK,
    )
    .await?;
    if !contains_id(&mine, own_id) {
        return Err("已审列表没有这份申报".to_owned());
    }
    expect_status(
        app,
        Method::GET,
        &format!("/api/department-reviews/{own_id}"),
        Some(reviewer),
        None,
        StatusCode::OK,
    )
    .await?;
    Ok(())
}

fn decision(kind: &str, opinion: &str, ready: bool) -> Value {
    json!({
        "decision": kind,
        "opinion": opinion,
        "facts_accurate": ready,
        "team_qualified": ready,
        "resources_ready": ready,
        "agree_to_manage": ready,
    })
}

fn contains_id(body: &Value, id: Uuid) -> bool {
    let text = id.to_string();
    body["items"].as_array().is_some_and(|items| {
        items
            .iter()
            .any(|item| item["id"].as_str() == Some(text.as_str()))
    })
}

async fn insert_submitted(
    state: &AppState,
    cycle_id: Uuid,
    principal_id: Uuid,
    department_id: Uuid,
    title: &str,
) -> Uuid {
    let id = Uuid::new_v4();
    let timestamp = now();
    let model = applications::ActiveModel {
        id: Set(id),
        cycle_id: Set(cycle_id),
        project_class: Set(ProjectClass::Social),
        title: Set(Some(title.to_owned())),
        principal_id: Set(principal_id),
        principal_name: Set(Some("接口测试负责人".to_owned())),
        professional_title: Set(Some("讲师".to_owned())),
        department_id: Set(Some(department_id)),
        status: Set(ApplicationStatus::Submitted),
        commitment_accepted: Set(true),
        commitment_accepted_at: Set(Some(timestamp)),
        submitted_at: Set(Some(timestamp)),
        created_at: Set(timestamp),
        updated_at: Set(timestamp),
        ..Default::default()
    };
    model
        .insert(&state.db)
        .await
        .unwrap_or_else(|error| panic!("{error}"));
    id
}

async fn user_by_name(state: &AppState, username: &str) -> users::Model {
    users::Entity::find()
        .filter(users::Column::Username.eq(username))
        .one(&state.db)
        .await
        .unwrap_or_else(|error| panic!("{error}"))
        .unwrap_or_else(|| panic!("缺少演示账号 {username}"))
}

async fn token(state: &AppState, username: &str) -> String {
    let user = user_by_name(state, username).await;
    encode_session(user.id, &state.config.jwt_secret).unwrap_or_else(|error| panic!("{error}"))
}

async fn other_department(state: &AppState, current: Uuid) -> Uuid {
    departments::Entity::find()
        .filter(departments::Column::Id.ne(current))
        .one(&state.db)
        .await
        .unwrap_or_else(|error| panic!("{error}"))
        .map(|row| row.id)
        .unwrap_or_else(|| panic!("缺少第二个部门"))
}

async fn expect_status(
    app: &Router,
    method: Method,
    uri: &str,
    token: Option<&str>,
    body: Option<Value>,
    expected: StatusCode,
) -> Result<Value, String> {
    let mut request = Request::builder().method(method).uri(uri);
    if let Some(token) = token {
        request = request.header("authorization", format!("Bearer {token}"));
    }
    let content = match body {
        Some(value) => {
            request = request.header("content-type", "application/json");
            value.to_string()
        }
        None => String::new(),
    };
    let request = request
        .body(Body::from(content))
        .map_err(|error| error.to_string())?;
    let response = app
        .clone()
        .oneshot(request)
        .await
        .map_err(|error| error.to_string())?;
    let status = response.status();
    let bytes = response
        .into_body()
        .collect()
        .await
        .map_err(|error| error.to_string())?
        .to_bytes();
    let value = if bytes.is_empty() {
        Value::Null
    } else {
        serde_json::from_slice(&bytes).map_err(|error| error.to_string())?
    };
    if status != expected {
        return Err(format!(
            "{uri} 期望 {expected}，实际 {status}，响应 {value}"
        ));
    }
    Ok(value)
}

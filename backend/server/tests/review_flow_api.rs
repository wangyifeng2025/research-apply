use axum::body::Body;
use axum::http::{Method, Request, StatusCode};
use entity::sea_orm_active_enums::{ApplicationStatus, ProjectClass};
use entity::{application_cycles, applications, users};
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
async fn office_assigns_experts_and_conclusion_stays_anonymous() {
    dotenvy::dotenv().ok();
    let config = Config::from_env().unwrap_or_else(|error| panic!("{error}"));
    let state = AppState::connect(&config)
        .await
        .unwrap_or_else(|error| panic!("{error}"));
    let app = routes::router(state.clone(), &config).unwrap_or_else(|error| panic!("{error}"));
    let teacher = user_by_name(&state, "zhaomingyuan").await;
    let reviewer = user_by_name(&state, "liwenbo").await;
    let office_user = user_by_name(&state, "wanghaining").await;
    let expert = user_by_name(&state, "expert-e016").await;
    let department = reviewer
        .department_id
        .unwrap_or_else(|| panic!("系审核账号没有部门"));
    let cycle = application_cycles::Entity::find()
        .one(&state.db)
        .await
        .unwrap_or_else(|error| panic!("{error}"))
        .unwrap_or_else(|| panic!("缺少申报批次"));
    let first =
        insert_submitted(&state, cycle.id, teacher.id, department, "科研处流程测试甲").await;
    let second =
        insert_submitted(&state, cycle.id, teacher.id, department, "科研处流程测试乙").await;
    let conflict = insert_principal(
        &state,
        cycle.id,
        expert.id,
        department,
        "专家回避测试",
        "匿名专家E-016",
    )
    .await;

    let result = exercise(
        &app,
        &Actors {
            teacher: &token(&state, teacher.id).await,
            reviewer: &token(&state, reviewer.id).await,
            office: &token(&state, office_user.id).await,
            expert: &token(&state, expert.id).await,
            office_id: office_user.id,
            expert_id: expert.id,
            expert_name: &expert.real_name,
        },
        Apps {
            first,
            second,
            conflict,
        },
    )
    .await;
    for id in [first, second, conflict] {
        let _ = applications::Entity::delete_by_id(id).exec(&state.db).await;
    }
    if let Err(error) = result {
        panic!("{error}");
    }
}

struct Actors<'a> {
    teacher: &'a str,
    reviewer: &'a str,
    office: &'a str,
    expert: &'a str,
    office_id: Uuid,
    expert_id: Uuid,
    expert_name: &'a str,
}

struct Apps {
    first: Uuid,
    second: Uuid,
    conflict: Uuid,
}

async fn exercise(app: &axum::Router, actors: &Actors<'_>, apps: Apps) -> Result<(), String> {
    let Apps {
        first,
        second,
        conflict,
    } = apps;
    for id in [first, second, conflict] {
        expect_status(
            app,
            actors.reviewer,
            "POST",
            &format!("/api/department-reviews/{id}"),
            Some(checks()),
        )
        .await?;
    }
    expect_status(
        app,
        actors.office,
        "POST",
        &format!("/api/office-reviews/{first}"),
        Some(approve("KY-FLOW-A")),
    )
    .await?;
    let duplicate = call(
        app,
        actors.office,
        "POST",
        &format!("/api/office-reviews/{second}"),
        Some(approve("KY-FLOW-A")),
    )
    .await?;
    if duplicate.status != StatusCode::CONFLICT {
        return Err(format!("重复封面编号应返回 409，实际 {}", duplicate.status));
    }
    expect_status(
        app,
        actors.office,
        "POST",
        &format!("/api/office-reviews/{second}"),
        Some(approve("KY-FLOW-B")),
    )
    .await?;
    expect_status(
        app,
        actors.office,
        "POST",
        &format!("/api/office-reviews/{conflict}"),
        Some(approve("KY-FLOW-C")),
    )
    .await?;

    let self_assign = call(
        app,
        actors.office,
        "POST",
        &format!("/api/office-reviews/{first}/assignments"),
        Some(json!({ "expert_ids": [actors.office_id] })),
    )
    .await?;
    if self_assign.status != StatusCode::UNPROCESSABLE_ENTITY {
        return Err(format!("指派自己应返回 422，实际 {}", self_assign.status));
    }
    let conflict_assign = call(
        app,
        actors.office,
        "POST",
        &format!("/api/office-reviews/{conflict}/assignments"),
        Some(json!({ "expert_ids": [actors.expert_id] })),
    )
    .await?;
    let conflict_message = conflict_assign.body["fields"]
        .as_array()
        .and_then(|fields| fields.first())
        .and_then(|field| field["message"].as_str())
        .unwrap_or("");
    if conflict_assign.status != StatusCode::UNPROCESSABLE_ENTITY
        || !conflict_message.contains("负责人")
    {
        return Err(format!(
            "指派负责人应被拒绝，实际 {} {}",
            conflict_assign.status, conflict_assign.body
        ));
    }
    expect_status(
        app,
        actors.office,
        "POST",
        &format!("/api/office-reviews/{first}/assignments"),
        Some(json!({ "expert_ids": [actors.expert_id] })),
    )
    .await?;

    let blind = call(
        app,
        actors.expert,
        "GET",
        &format!("/api/expert-reviews/{first}"),
        None,
    )
    .await?;
    if blind.status != StatusCode::OK {
        return Err(format!("专家读取应成功，实际 {}", blind.status));
    }
    let serialized = blind.body.to_string();
    for leaked in [
        "principal_name",
        "department_name",
        "members",
        "recommender",
        "赵明远",
        "reviewer_id",
    ] {
        if serialized.contains(leaked) {
            return Err(format!("匿名申报书包含 {leaked}"));
        }
    }
    expect_status(
        app,
        actors.expert,
        "POST",
        &format!("/api/expert-reviews/{first}"),
        Some(json!({ "decision": "approved", "opinion": "研究内容完整", "score": "88.50" })),
    )
    .await?;
    let repeated = call(
        app,
        actors.expert,
        "POST",
        &format!("/api/expert-reviews/{first}"),
        Some(json!({ "decision": "approved", "opinion": "再次提交", "score": "90" })),
    )
    .await?;
    if repeated.status != StatusCode::CONFLICT {
        return Err(format!("重复评审应返回 409，实际 {}", repeated.status));
    }
    expect_status(
        app,
        actors.office,
        "POST",
        &format!("/api/office-reviews/{first}/conclusion"),
        Some(json!({ "decision": "established", "opinion": "同意立项" })),
    )
    .await?;
    let mine = call(app, actors.teacher, "GET", "/api/applications/mine", None).await?;
    let item = mine.body["items"]
        .as_array()
        .and_then(|items| {
            items
                .iter()
                .find(|item| item["id"] == json!(first.to_string()))
        })
        .cloned()
        .ok_or("申报人列表没有这笔立项申报")?;
    if item["conclusion_opinion"] != "同意立项" || item.to_string().contains("reviewer_id") {
        return Err(format!("申报人应看到结论说明且看不到专家身份：{item}"));
    }
    let teacher_timeline = call(
        app,
        actors.teacher,
        "GET",
        &format!("/api/applications/{first}/timeline"),
        None,
    )
    .await?;
    let teacher_text = teacher_timeline.body.to_string();
    if teacher_timeline.status != StatusCode::OK
        || !teacher_text.contains("专家 1")
        || !teacher_text.contains("同意立项")
        || !teacher_text.contains("88.50")
        || teacher_text.contains(actors.expert_name)
        || teacher_text.contains("reviewer_id")
    {
        return Err(format!(
            "申报人时间线应匿名展示专家：{} {}",
            teacher_timeline.status, teacher_timeline.body
        ));
    }
    let office_timeline = call(
        app,
        actors.office,
        "GET",
        &format!("/api/applications/{first}/timeline"),
        None,
    )
    .await?;
    if office_timeline.status != StatusCode::OK
        || !office_timeline
            .body
            .to_string()
            .contains(actors.expert_name)
    {
        return Err(format!(
            "科研处时间线应看到专家姓名：{} {}",
            office_timeline.status, office_timeline.body
        ));
    }
    let hidden = call(
        app,
        actors.expert,
        "GET",
        &format!("/api/applications/{first}/timeline"),
        None,
    )
    .await?;
    if hidden.status != StatusCode::FORBIDDEN {
        return Err(format!(
            "未授权专家读取时间线应返回 403，实际 {}",
            hidden.status
        ));
    }
    let foreign = call(
        app,
        actors.teacher,
        "GET",
        &format!("/api/applications/{conflict}/timeline"),
        None,
    )
    .await?;
    if foreign.status != StatusCode::FORBIDDEN {
        return Err(format!(
            "教师不能查看他人申报书的时间线，实际 {}",
            foreign.status
        ));
    }
    Ok(())
}

fn checks() -> Value {
    json!({
        "decision": "approved",
        "opinion": "同意上报",
        "facts_accurate": true,
        "team_qualified": true,
        "resources_ready": true,
        "agree_to_manage": true
    })
}

fn approve(code: &str) -> Value {
    json!({ "decision": "approved", "opinion": "同意进入专家评审", "code": code })
}

async fn expect_status(
    app: &axum::Router,
    token: &str,
    method: &str,
    path: &str,
    body: Option<Value>,
) -> Result<(), String> {
    let response = call(app, token, method, path, body).await?;
    if response.status == StatusCode::OK {
        Ok(())
    } else {
        Err(format!(
            "{method} {path} 返回 {} {}",
            response.status, response.body
        ))
    }
}

struct ResponseBody {
    status: StatusCode,
    body: Value,
}

async fn call(
    app: &axum::Router,
    token: &str,
    method: &str,
    path: &str,
    body: Option<Value>,
) -> Result<ResponseBody, String> {
    let method = match method {
        "GET" => Method::GET,
        "POST" => Method::POST,
        _ => return Err(format!("不支持的方法 {method}")),
    };
    let request = Request::builder()
        .method(method)
        .uri(path)
        .header("authorization", format!("Bearer {token}"))
        .header("content-type", "application/json")
        .body(Body::from(
            body.map(|value| value.to_string()).unwrap_or_default(),
        ))
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
    let body = serde_json::from_slice(&bytes).unwrap_or(Value::Null);
    Ok(ResponseBody { status, body })
}

async fn token(state: &AppState, user_id: Uuid) -> String {
    encode_session(user_id, &state.config.jwt_secret).unwrap_or_else(|error| panic!("{error}"))
}

async fn user_by_name(state: &AppState, username: &str) -> users::Model {
    users::Entity::find()
        .filter(users::Column::Username.eq(username))
        .one(&state.db)
        .await
        .unwrap_or_else(|error| panic!("{error}"))
        .unwrap_or_else(|| panic!("缺少演示账号 {username}"))
}

async fn insert_submitted(
    state: &AppState,
    cycle_id: Uuid,
    principal_id: Uuid,
    department_id: Uuid,
    title: &str,
) -> Uuid {
    insert_principal(
        state,
        cycle_id,
        principal_id,
        department_id,
        title,
        "赵明远",
    )
    .await
}

async fn insert_principal(
    state: &AppState,
    cycle_id: Uuid,
    principal_id: Uuid,
    department_id: Uuid,
    title: &str,
    principal_name: &str,
) -> Uuid {
    let timestamp = now();
    applications::ActiveModel {
        cycle_id: Set(cycle_id),
        project_class: Set(ProjectClass::Social),
        status: Set(ApplicationStatus::Submitted),
        principal_id: Set(principal_id),
        department_id: Set(Some(department_id)),
        principal_name: Set(Some(principal_name.to_owned())),
        professional_title: Set(Some("讲师".to_owned())),
        title: Set(Some(title.to_owned())),
        submitted_at: Set(Some(timestamp)),
        commitment_accepted: Set(true),
        commitment_accepted_at: Set(Some(timestamp)),
        created_at: Set(timestamp),
        updated_at: Set(timestamp),
        ..Default::default()
    }
    .insert(&state.db)
    .await
    .unwrap_or_else(|error| panic!("{error}"))
    .id
}

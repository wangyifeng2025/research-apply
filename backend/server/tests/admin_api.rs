use axum::Router;
use axum::body::Body;
use axum::http::{Method, Request, StatusCode};
use entity::sea_orm_active_enums::CycleStatus;
use entity::{application_cycles, applications, departments, users};
use http_body_util::BodyExt;
use research_apply_server::auth::{encode_session, verify_password};
use research_apply_server::config::Config;
use research_apply_server::routes;
use research_apply_server::state::AppState;
use sea_orm::{ActiveModelTrait, ActiveValue::Set, ColumnTrait, EntityTrait, QueryFilter};
use serde_json::{Value, json};
use tower::ServiceExt;
use uuid::Uuid;

#[derive(Default)]
struct Created {
    department_id: Option<Uuid>,
    user_id: Option<Uuid>,
    cycle_id: Option<Uuid>,
    application_id: Option<Uuid>,
    cycles_to_reopen: Vec<Uuid>,
}

#[tokio::test]
#[ignore = "需要本地 PostgreSQL 和演示账号"]
async fn management_apis_cover_permissions_crud_and_cycle_controls() {
    dotenvy::dotenv().ok();
    let config = Config::from_env().unwrap_or_else(|error| panic!("{error}"));
    let state = AppState::connect(&config)
        .await
        .unwrap_or_else(|error| panic!("{error}"));
    let app = routes::router(state.clone(), &config).unwrap_or_else(|error| panic!("{error}"));
    let admin = token(&state, "zhoujin").await;
    let office = token(&state, "wanghaining").await;
    let teacher = token(&state, "zhaomingyuan").await;
    let mut created = Created::default();
    let result = exercise(&app, &state, &admin, &office, &teacher, &mut created).await;
    cleanup(&state, &created).await;
    if let Err(error) = result {
        panic!("{error}");
    }
}

async fn exercise(
    app: &Router,
    state: &AppState,
    admin: &str,
    office: &str,
    teacher: &str,
    created: &mut Created,
) -> Result<(), String> {
    expect_status(
        app,
        Method::GET,
        "/api/admin/users",
        None,
        None,
        StatusCode::UNAUTHORIZED,
    )
    .await?;
    expect_status(
        app,
        Method::POST,
        "/api/admin/departments",
        Some(teacher),
        Some(json!({ "name": "越权部门", "code": "FORBIDDEN" })),
        StatusCode::FORBIDDEN,
    )
    .await?;

    let suffix = Uuid::new_v4().simple().to_string();
    let name = format!("接口测试部门-{suffix}");
    let code = format!("T{}", &suffix[..12]);
    let department = expect_status(
        app,
        Method::POST,
        "/api/admin/departments",
        Some(admin),
        Some(json!({ "name": name, "code": code })),
        StatusCode::CREATED,
    )
    .await?;
    created.department_id = Some(uuid_field(&department, "id")?);
    expect_status(
        app,
        Method::POST,
        "/api/admin/departments",
        Some(admin),
        Some(json!({ "name": name, "code": code })),
        StatusCode::CONFLICT,
    )
    .await?;
    expect_status(
        app,
        Method::PATCH,
        &format!(
            "/api/admin/departments/{}",
            created.department_id.unwrap_or_default()
        ),
        Some(admin),
        Some(json!({ "name": format!("{name}-已修改"), "code": code })),
        StatusCode::OK,
    )
    .await?;

    let username = format!("api-test-{suffix}");
    let user = expect_status(
        app,
        Method::POST,
        "/api/admin/users",
        Some(admin),
        Some(json!({
            "username": username,
            "password": "initial-password",
            "real_name": "接口测试用户",
            "employee_no": format!("E{}", &suffix[..12]),
            "email": format!("{suffix}@example.test"),
            "department_id": created.department_id,
            "roles": ["teacher", "expert"]
        })),
        StatusCode::CREATED,
    )
    .await?;
    let user_id = uuid_field(&user, "id")?;
    created.user_id = Some(user_id);
    if user["roles"].as_array().map(Vec::len) != Some(2) {
        return Err("新建用户未保存两个角色".to_owned());
    }
    let filtered = expect_status(
        app,
        Method::GET,
        &format!("/api/admin/users?role=expert&keyword={username}"),
        Some(admin),
        None,
        StatusCode::OK,
    )
    .await?;
    if filtered["items"].as_array().map(Vec::len) != Some(1) {
        return Err("用户部门/角色/关键字筛选结果不正确".to_owned());
    }
    expect_status(
        app,
        Method::POST,
        "/api/admin/users",
        Some(admin),
        Some(json!({
            "username": username,
            "password": "another-password",
            "real_name": "重复账号",
            "roles": ["teacher"]
        })),
        StatusCode::CONFLICT,
    )
    .await?;
    let updated = expect_status(
        app,
        Method::PATCH,
        &format!("/api/admin/users/{user_id}"),
        Some(admin),
        Some(json!({ "real_name": "接口测试用户已修改", "employee_no": null, "roles": ["teacher", "expert"] })),
        StatusCode::OK,
    )
    .await?;
    if !updated["employee_no"].is_null() || updated["roles"].as_array().map(Vec::len) != Some(2) {
        return Err("用户修改未正确保存可空字段或多角色".to_owned());
    }
    expect_status(
        app,
        Method::POST,
        &format!("/api/admin/users/{user_id}/reset-password"),
        Some(admin),
        Some(json!({ "password": "reset-password" })),
        StatusCode::NO_CONTENT,
    )
    .await?;
    let stored_user = users::Entity::find_by_id(user_id)
        .one(&state.db)
        .await
        .map_err(|error| error.to_string())?
        .ok_or_else(|| "测试用户未写入数据库".to_owned())?;
    if !verify_password("reset-password", &stored_user.password_hash)
        .map_err(|error| error.to_string())?
    {
        return Err("重置后的密码不能通过校验".to_owned());
    }
    expect_status(
        app,
        Method::PATCH,
        &format!("/api/admin/users/{user_id}"),
        Some(admin),
        Some(json!({ "is_active": false })),
        StatusCode::OK,
    )
    .await?;
    let disabled_token =
        encode_session(user_id, &state.config.jwt_secret).map_err(|error| error.to_string())?;
    expect_status(
        app,
        Method::GET,
        "/api/applications/mine",
        Some(&disabled_token),
        None,
        StatusCode::FORBIDDEN,
    )
    .await?;
    expect_status(
        app,
        Method::DELETE,
        &format!(
            "/api/admin/departments/{}",
            created.department_id.unwrap_or_default()
        ),
        Some(admin),
        None,
        StatusCode::CONFLICT,
    )
    .await?;

    let cycle = expect_status(
        app,
        Method::POST,
        "/api/cycles",
        Some(office),
        Some(json!({ "cycle_year": 2090, "title": format!("接口测试批次-{suffix}"), "status": "open" })),
        StatusCode::CREATED,
    )
    .await?;
    let cycle_id = uuid_field(&cycle, "id")?;
    created.cycle_id = Some(cycle_id);
    expect_status(
        app,
        Method::PATCH,
        &format!("/api/cycles/{cycle_id}"),
        Some(admin),
        Some(json!({ "opens_at": "2090-01-02T00:00:00+00:00", "closes_at": "2090-01-01T00:00:00+00:00" })),
        StatusCode::UNPROCESSABLE_ENTITY,
    )
    .await?;
    expect_status(
        app,
        Method::GET,
        "/api/cycles",
        Some(office),
        None,
        StatusCode::OK,
    )
    .await?;
    expect_status(
        app,
        Method::GET,
        "/api/cycles/current",
        None,
        None,
        StatusCode::OK,
    )
    .await?;
    let application = expect_status(
        app,
        Method::POST,
        "/api/applications",
        Some(teacher),
        Some(json!({ "project_class": "social" })),
        StatusCode::CREATED,
    )
    .await?;
    let application_id = uuid_field(&application, "id")?;
    created.application_id = Some(application_id);
    let stored_application = applications::Entity::find_by_id(application_id)
        .one(&state.db)
        .await
        .map_err(|error| error.to_string())?
        .ok_or_else(|| "测试申报书未写入数据库".to_owned())?;
    if stored_application.cycle_id != cycle_id {
        return Err("新建申报书没有关联当前开放批次".to_owned());
    }
    expect_status(
        app,
        Method::PATCH,
        &format!("/api/cycles/{cycle_id}"),
        Some(office),
        Some(json!({ "status": "closed" })),
        StatusCode::OK,
    )
    .await?;
    let other_open_cycles = application_cycles::Entity::find()
        .filter(application_cycles::Column::Status.eq(CycleStatus::Open))
        .all(&state.db)
        .await
        .map_err(|error| error.to_string())?;
    for cycle in other_open_cycles {
        let mut model: application_cycles::ActiveModel = cycle.clone().into();
        model.status = Set(CycleStatus::Closed);
        model
            .update(&state.db)
            .await
            .map_err(|error| error.to_string())?;
        created.cycles_to_reopen.push(cycle.id);
    }
    expect_status(
        app,
        Method::POST,
        "/api/applications",
        Some(teacher),
        Some(json!({ "project_class": "social" })),
        StatusCode::CONFLICT,
    )
    .await?;
    expect_status(
        app,
        Method::POST,
        &format!("/api/applications/{application_id}/submit"),
        Some(teacher),
        Some(json!({ "commitment_accepted": true })),
        StatusCode::CONFLICT,
    )
    .await?;
    expect_status(
        app,
        Method::PATCH,
        &format!("/api/cycles/{cycle_id}"),
        Some(office),
        Some(json!({ "status": "open" })),
        StatusCode::OK,
    )
    .await?;
    expect_status(
        app,
        Method::POST,
        &format!("/api/applications/{application_id}/submit"),
        Some(teacher),
        Some(json!({ "commitment_accepted": true })),
        StatusCode::UNPROCESSABLE_ENTITY,
    )
    .await?;
    expect_status(
        app,
        Method::DELETE,
        &format!("/api/applications/{application_id}"),
        Some(teacher),
        None,
        StatusCode::NO_CONTENT,
    )
    .await?;
    created.application_id = None;

    expect_status(
        app,
        Method::DELETE,
        &format!("/api/admin/users/{user_id}"),
        Some(admin),
        None,
        StatusCode::NO_CONTENT,
    )
    .await?;
    created.user_id = None;
    expect_status(
        app,
        Method::DELETE,
        &format!(
            "/api/admin/departments/{}",
            created.department_id.unwrap_or_default()
        ),
        Some(admin),
        None,
        StatusCode::NO_CONTENT,
    )
    .await?;
    created.department_id = None;
    Ok(())
}

async fn cleanup(state: &AppState, created: &Created) {
    if let Some(id) = created.application_id {
        let _ = applications::Entity::delete_by_id(id).exec(&state.db).await;
    }
    if let Some(id) = created.cycle_id {
        let _ = application_cycles::Entity::delete_by_id(id)
            .exec(&state.db)
            .await;
    }
    if let Some(id) = created.user_id {
        let _ = users::Entity::delete_by_id(id).exec(&state.db).await;
    }
    if let Some(id) = created.department_id {
        let _ = departments::Entity::delete_by_id(id).exec(&state.db).await;
    }
    for id in &created.cycles_to_reopen {
        if let Ok(Some(cycle)) = application_cycles::Entity::find_by_id(*id)
            .one(&state.db)
            .await
        {
            let mut model: application_cycles::ActiveModel = cycle.into();
            model.status = Set(CycleStatus::Open);
            let _ = model.update(&state.db).await;
        }
    }
}

async fn token(state: &AppState, username: &str) -> String {
    let user = users::Entity::find()
        .filter(users::Column::Username.eq(username))
        .one(&state.db)
        .await
        .unwrap_or_else(|error| panic!("{error}"))
        .unwrap_or_else(|| panic!("缺少演示账号 {username}"));
    encode_session(user.id, &state.config.jwt_secret).unwrap_or_else(|error| panic!("{error}"))
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

fn uuid_field(body: &Value, field: &str) -> Result<Uuid, String> {
    let value = body[field]
        .as_str()
        .ok_or_else(|| format!("响应缺少 {field}"))?;
    Uuid::parse_str(value).map_err(|error| error.to_string())
}

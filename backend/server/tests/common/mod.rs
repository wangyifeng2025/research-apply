//! 独立测试库。
//!
//! 从 `DATABASE_URL` 推导出以 `_test` 结尾的库名，不存在就创建，然后执行迁移。
//! 每个用例清掉业务数据后重新写入固定账号，不碰演示库里的申报。
//! 同一进程里的用例串行执行，避免互相清表。

use std::future::Future;
use std::net::SocketAddr;
use std::sync::OnceLock;

use axum::Router;
use axum::body::Body;
use axum::extract::connect_info::MockConnectInfo;
use axum::http::{HeaderMap, Method, Request, StatusCode, header};
use chrono::NaiveDate;
use entity::application_cycles;
use entity::departments;
use entity::sea_orm_active_enums::{CycleStatus, UserRole};
use entity::user_roles;
use entity::users;
use http_body_util::BodyExt;
use migration::Migrator;
use migration::MigratorTrait;
use research_apply_server::auth::hash_password;
use research_apply_server::config::{AUTH_COOKIE, Config};
use research_apply_server::routes;
use research_apply_server::state::{AppState, connect_database};
use research_apply_server::support::time::now;
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, ConnectionTrait, Database, DatabaseBackend,
    EntityTrait, QueryFilter, Statement,
};
use serde_json::Value;
use tokio::sync::Mutex;
use tower::ServiceExt;
use uuid::Uuid;

pub const PASSWORD: &str = "test-password-2026";

static GATE: OnceLock<Mutex<()>> = OnceLock::new();
static PASSWORD_HASH: OnceLock<String> = OnceLock::new();

pub struct TestApp {
    pub app: Router,
    pub teacher: String,
    pub other: String,
    pub office: String,
    pub admin: String,
}

pub async fn with_test_app<F, Fut>(case: F)
where
    F: FnOnce(TestApp) -> Fut,
    Fut: Future<Output = Result<(), String>>,
{
    let _guard = gate().lock().await;
    let app = prepare().await.unwrap_or_else(|error| panic!("{error}"));
    if let Err(error) = case(app).await {
        panic!("{error}");
    }
}

pub async fn call(
    app: &Router,
    method: Method,
    path: &str,
    token: Option<&str>,
    body: Option<Value>,
) -> Result<(StatusCode, Value), String> {
    let (status, bytes) = call_bytes(app, method, path, token, body).await?;
    if bytes.is_empty() {
        return Ok((status, Value::Null));
    }
    let value = serde_json::from_slice(&bytes).unwrap_or(Value::Null);
    Ok((status, value))
}

pub async fn call_bytes(
    app: &Router,
    method: Method,
    path: &str,
    token: Option<&str>,
    body: Option<Value>,
) -> Result<(StatusCode, Vec<u8>), String> {
    let mut builder = Request::builder().method(method).uri(path);
    if let Some(token) = token {
        builder = builder.header(header::AUTHORIZATION, format!("Bearer {token}"));
    }
    let request = if let Some(body) = body {
        builder
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(
                serde_json::to_vec(&body).map_err(|error| error.to_string())?,
            ))
            .map_err(|error| error.to_string())?
    } else {
        builder
            .body(Body::empty())
            .map_err(|error| error.to_string())?
    };
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
        .to_bytes()
        .to_vec();
    Ok((status, bytes))
}

pub async fn login(
    app: &Router,
    username: &str,
    password: &str,
) -> Result<(StatusCode, Value, Option<String>), String> {
    let request = Request::builder()
        .method(Method::POST)
        .uri("/api/auth/login")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            serde_json::to_vec(&serde_json::json!({
                "username": username,
                "password": password
            }))
            .map_err(|error| error.to_string())?,
        ))
        .map_err(|error| error.to_string())?;
    let response = app
        .clone()
        .oneshot(request)
        .await
        .map_err(|error| error.to_string())?;
    let status = response.status();
    let token = session_token(response.headers());
    let bytes = response
        .into_body()
        .collect()
        .await
        .map_err(|error| error.to_string())?
        .to_bytes();
    let value = serde_json::from_slice(&bytes).unwrap_or(Value::Null);
    Ok((status, value, token))
}

fn gate() -> &'static Mutex<()> {
    GATE.get_or_init(|| Mutex::new(()))
}

fn password_hash() -> &'static str {
    PASSWORD_HASH.get_or_init(|| {
        hash_password(PASSWORD).unwrap_or_else(|error| panic!("测试密码哈希失败: {error}"))
    })
}

async fn prepare() -> Result<TestApp, String> {
    load_env();
    let source = std::env::var("DATABASE_URL")
        .map_err(|_| "缺少 DATABASE_URL。请在 backend/.env 配置后再运行这个测试".to_owned())?;
    let test_url = test_database_url(&source)?;
    let name = database_name(&test_url)?;
    if !name.ends_with("_test") {
        return Err(format!("拒绝在非测试库执行清表: {name}"));
    }
    let admin_url = replace_database(&test_url, "postgres")?;
    ensure_database(&admin_url, &name).await?;
    let db = connect_database(&test_url)
        .await
        .map_err(|error| error.to_string())?;
    Migrator::up(&db, None)
        .await
        .map_err(|error| error.to_string())?;
    reset_business_data(&db).await?;
    seed(&db).await?;
    let config = Config {
        database_url: test_url,
        bind: "127.0.0.1:0".to_owned(),
        cors_origin: "http://localhost:3000".to_owned(),
        jwt_secret: "teacher-flow-test-secret".to_owned(),
        cookie_secure: false,
        attachment_dir: std::env::temp_dir().join("research-apply-test-attachments"),
    };
    let state = AppState {
        db: db.clone(),
        config: config.clone(),
    };
    let app = routes::router(state, &config)
        .map_err(|error| error.to_string())?
        .layer(MockConnectInfo(SocketAddr::from(([127, 0, 0, 1], 18080))));
    let teacher = token_for(&app, "flow-teacher").await?;
    let other = token_for(&app, "flow-other").await?;
    let office = token_for(&app, "flow-office").await?;
    let admin = token_for(&app, "flow-admin").await?;
    Ok(TestApp {
        app,
        teacher,
        other,
        office,
        admin,
    })
}

async fn token_for(app: &Router, username: &str) -> Result<String, String> {
    let (status, body, token) = login(app, username, PASSWORD).await?;
    if status != StatusCode::OK {
        return Err(format!("{username} 登录失败: {status} {body}"));
    }
    token.ok_or_else(|| format!("{username} 登录响应没有会话 Cookie"))
}

async fn seed(db: &sea_orm::DatabaseConnection) -> Result<(), String> {
    let moment = now();
    let department_id = Uuid::new_v4();
    departments::ActiveModel {
        id: Set(department_id),
        name: Set("接口测试学院".to_owned()),
        code: Set(Some("FLOWTEST".to_owned())),
        created_at: Set(moment),
        updated_at: Set(moment),
    }
    .insert(db)
    .await
    .map_err(|error| error.to_string())?;
    insert_user(
        db,
        "flow-teacher",
        "流程教师",
        Some(department_id),
        UserRole::Teacher,
        true,
        true,
    )
    .await?;
    insert_user(
        db,
        "flow-other",
        "另一位教师",
        Some(department_id),
        UserRole::Teacher,
        true,
        false,
    )
    .await?;
    insert_user(
        db,
        "flow-office",
        "流程科研处",
        None,
        UserRole::ResearchOffice,
        true,
        false,
    )
    .await?;
    insert_user(
        db,
        "flow-admin",
        "流程管理员",
        None,
        UserRole::Admin,
        true,
        false,
    )
    .await?;
    insert_user(
        db,
        "flow-inactive",
        "已停用教师",
        Some(department_id),
        UserRole::Teacher,
        false,
        false,
    )
    .await?;
    let cycle = application_cycles::Entity::find()
        .filter(application_cycles::Column::CycleYear.eq(2026))
        .one(db)
        .await
        .map_err(|error| error.to_string())?
        .ok_or_else(|| "测试库缺少 2026 批次".to_owned())?;
    let mut active: application_cycles::ActiveModel = cycle.into();
    active.status = Set(CycleStatus::Open);
    active.opens_at = Set(Some(moment - chrono::Duration::days(1)));
    active.closes_at = Set(Some(moment + chrono::Duration::days(60)));
    active.updated_at = Set(moment);
    active.update(db).await.map_err(|error| error.to_string())?;
    Ok(())
}

async fn insert_user(
    db: &sea_orm::DatabaseConnection,
    username: &str,
    real_name: &str,
    department_id: Option<Uuid>,
    role: UserRole,
    active: bool,
    with_profile: bool,
) -> Result<Uuid, String> {
    let moment = now();
    let id = Uuid::new_v4();
    users::ActiveModel {
        id: Set(id),
        username: Set(username.to_owned()),
        password_hash: Set(password_hash().to_owned()),
        real_name: Set(real_name.to_owned()),
        employee_no: Set(Some(username.to_owned())),
        gender: Set(with_profile.then(|| "男".to_owned())),
        ethnicity: Set(with_profile.then(|| "汉族".to_owned())),
        birth_date: Set(with_profile
            .then(|| NaiveDate::from_ymd_opt(1985, 6, 1))
            .flatten()),
        admin_title: Set(with_profile.then(|| "教研室主任".to_owned())),
        professional_title: Set(with_profile.then(|| "副教授".to_owned())),
        research_expertise: Set(with_profile.then(|| "区域经济".to_owned())),
        education: Set(with_profile.then(|| "研究生".to_owned())),
        degree: Set(with_profile.then(|| "硕士".to_owned())),
        phone: Set(with_profile.then(|| "13700001001".to_owned())),
        email: Set(Some(format!("{username}@example.test"))),
        department_id: Set(department_id),
        is_active: Set(active),
        created_at: Set(moment),
        updated_at: Set(moment),
    }
    .insert(db)
    .await
    .map_err(|error| error.to_string())?;
    user_roles::ActiveModel {
        user_id: Set(id),
        role: Set(role),
    }
    .insert(db)
    .await
    .map_err(|error| error.to_string())?;
    Ok(id)
}

async fn reset_business_data(db: &sea_orm::DatabaseConnection) -> Result<(), String> {
    db.execute_raw(Statement::from_string(
        DatabaseBackend::Postgres,
        "TRUNCATE TABLE
            audit_logs,
            notifications,
            reviews,
            expert_assignments,
            application_attachments,
            application_budgets,
            application_members,
            recommenders,
            ss_prior_achievements,
            ss_designs,
            ss_foundations,
            ss_expected_outputs,
            ns_profiles,
            ns_research_contents,
            ns_indicators,
            ns_schedules,
            ns_equipments,
            applications,
            user_roles,
            users,
            departments
        RESTART IDENTITY CASCADE"
            .to_owned(),
    ))
    .await
    .map_err(|error| error.to_string())?;
    Ok(())
}

async fn ensure_database(admin_url: &str, name: &str) -> Result<(), String> {
    let admin = Database::connect(admin_url)
        .await
        .map_err(|error| error.to_string())?;
    let existing = admin
        .query_one_raw(Statement::from_sql_and_values(
            DatabaseBackend::Postgres,
            "SELECT 1 FROM pg_database WHERE datname = $1",
            [name.into()],
        ))
        .await
        .map_err(|error| error.to_string())?;
    if existing.is_none() {
        admin
            .execute_unprepared(&format!("CREATE DATABASE \"{name}\""))
            .await
            .map_err(|error| error.to_string())?;
    }
    admin.close().await.map_err(|error| error.to_string())?;
    Ok(())
}

fn load_env() {
    for path in [".env", "backend/.env", "../.env"] {
        dotenvy::from_path(path).ok();
    }
}

fn test_database_url(raw: &str) -> Result<String, String> {
    if let Ok(explicit) = std::env::var("TEST_DATABASE_URL") {
        if !explicit.trim().is_empty() {
            return Ok(explicit);
        }
    }
    let name = database_name(raw)?;
    let test_name = if name.ends_with("_test") {
        name
    } else {
        format!("{name}_test")
    };
    replace_database(raw, &test_name)
}

fn replace_database(raw: &str, database: &str) -> Result<String, String> {
    let (base, query) = match raw.split_once('?') {
        Some((base, query)) => (base, Some(query)),
        None => (raw, None),
    };
    let (head, _) = base
        .rsplit_once('/')
        .ok_or_else(|| "DATABASE_URL 格式不正确".to_owned())?;
    let mut url = format!("{head}/{database}");
    if let Some(query) = query {
        url.push('?');
        url.push_str(query);
    }
    Ok(url)
}

fn database_name(url: &str) -> Result<String, String> {
    let base = url.split('?').next().unwrap_or(url);
    let name = base.rsplit('/').next().unwrap_or("").to_owned();
    if name.is_empty()
        || !name
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || ch == '_')
    {
        return Err(format!("数据库名无效: {name}"));
    }
    Ok(name)
}

fn session_token(headers: &HeaderMap) -> Option<String> {
    let prefix = format!("{AUTH_COOKIE}=");
    headers
        .get_all(header::SET_COOKIE)
        .iter()
        .find_map(|value| {
            let text = value.to_str().ok()?;
            let rest = text.strip_prefix(&prefix)?;
            let token = rest.split(';').next()?.trim();
            (!token.is_empty()).then(|| token.to_owned())
        })
}

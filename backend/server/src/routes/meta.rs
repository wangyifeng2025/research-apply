use axum::Json;
use axum::extract::State;
use entity::budget_subjects::Entity as BudgetSubject;
use sea_orm::{
    ConnectionTrait, DatabaseBackend, DatabaseConnection, EntityTrait, PaginatorTrait, QueryResult,
    Statement,
};
use serde::Serialize;

use crate::error::AppError;
use crate::state::AppState;

#[derive(Serialize)]
pub struct MetaResponse {
    service: &'static str,
    database: String,
    workflow: Vec<WorkflowStep>,
    project_classes: Vec<ProjectClass>,
    cycle: Option<CycleInfo>,
    budget_subject_count: u64,
}

#[derive(Serialize)]
struct WorkflowStep {
    code: &'static str,
    label: &'static str,
}

#[derive(Serialize)]
struct ProjectClass {
    code: &'static str,
    label: &'static str,
    form: &'static str,
}

#[derive(Serialize)]
struct CycleInfo {
    year: i32,
    title: String,
    status: String,
}

pub async fn meta(State(state): State<AppState>) -> Result<Json<MetaResponse>, AppError> {
    let database = text_value(&state.db, "SELECT current_database() AS value").await?;
    let budget_subject_count = BudgetSubject::find().count(&state.db).await?;
    let cycle = load_cycle(&state.db).await?;

    Ok(Json(MetaResponse {
        service: "research-apply",
        database,
        workflow: vec![
            WorkflowStep {
                code: "teacher",
                label: "教师填报",
            },
            WorkflowStep {
                code: "department",
                label: "系审核",
            },
            WorkflowStep {
                code: "research_office",
                label: "科研处审核",
            },
            WorkflowStep {
                code: "expert",
                label: "专家盲审",
            },
        ],
        project_classes: vec![
            ProjectClass {
                code: "social",
                label: "B.社会科学类",
                form: "社会科学类项目申报书",
            },
            ProjectClass {
                code: "natural",
                label: "A.自然科学类",
                form: "自然科学类项目申报书",
            },
            ProjectClass {
                code: "dual_high",
                label: "D.双高校建设专项",
                form: "自然科学类项目申报书",
            },
        ],
        cycle,
        budget_subject_count,
    }))
}

async fn load_cycle(db: &DatabaseConnection) -> Result<Option<CycleInfo>, AppError> {
    let row = query_one(
        db,
        "SELECT cycle_year, title, status::text AS status \
         FROM application_cycles \
         ORDER BY cycle_year DESC \
         LIMIT 1",
    )
    .await?;

    let Some(row) = row else {
        return Ok(None);
    };

    Ok(Some(CycleInfo {
        year: row.try_get("", "cycle_year")?,
        title: row.try_get("", "title")?,
        status: row.try_get("", "status")?,
    }))
}

async fn query_one(db: &DatabaseConnection, sql: &str) -> Result<Option<QueryResult>, AppError> {
    Ok(db
        .query_one_raw(Statement::from_string(DatabaseBackend::Postgres, sql))
        .await?)
}

async fn text_value(db: &DatabaseConnection, sql: &str) -> Result<String, AppError> {
    let row = required_row(db, sql).await?;
    Ok(row.try_get("", "value")?)
}

async fn required_row(db: &DatabaseConnection, sql: &str) -> Result<QueryResult, AppError> {
    query_one(db, sql)
        .await?
        .ok_or_else(|| sea_orm::DbErr::Custom("查询没有返回结果".to_owned()).into())
}

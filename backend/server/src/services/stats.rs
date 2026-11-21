//! 科研处统计：按批次汇总申报数量和申请经费。

use std::collections::HashMap;

use entity::sea_orm_active_enums::{ApplicationStatus, UserRole};
use entity::{application_budgets, application_cycles, applications, budget_subjects, departments};
use rust_xlsxwriter::{Format, Workbook};
use sea_orm::prelude::Decimal;
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::auth::{CurrentUser, require_role};
use crate::error::AppError;
use crate::services::applications::{budget_total, class_code, family_of, money_text, status_code};
use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct StatsQuery {
    pub cycle_id: Option<Uuid>,
}

#[derive(Debug, Serialize)]
pub struct CycleBrief {
    pub id: Uuid,
    pub cycle_year: i32,
    pub title: String,
    pub status: String,
}

#[derive(Debug, Serialize)]
pub struct StatsBucket {
    pub code: String,
    pub label: String,
    pub count: i64,
    pub amount: String,
}

#[derive(Debug, Serialize)]
pub struct StatsOverview {
    pub cycle: Option<CycleBrief>,
    pub cycles: Vec<CycleBrief>,
    pub total_count: i64,
    pub total_amount: String,
    pub by_class: Vec<StatsBucket>,
    pub by_department: Vec<StatsBucket>,
    pub by_status: Vec<StatsBucket>,
}

pub async fn overview(
    state: &AppState,
    actor: &CurrentUser,
    query: StatsQuery,
) -> Result<StatsOverview, AppError> {
    require_role(actor, UserRole::ResearchOffice)?;
    let cycles = application_cycles::Entity::find()
        .order_by_desc(application_cycles::Column::CycleYear)
        .order_by_desc(application_cycles::Column::CreatedAt)
        .all(&state.db)
        .await?;
    let briefs = cycles.iter().map(cycle_brief).collect::<Vec<_>>();
    let selected = match query.cycle_id {
        Some(id) => Some(
            cycles
                .iter()
                .find(|cycle| cycle.id == id)
                .ok_or_else(|| AppError::NotFound("申报批次不存在".to_owned()))?,
        ),
        None => cycles
            .iter()
            .find(|cycle| cycle.status == entity::sea_orm_active_enums::CycleStatus::Open)
            .or(cycles.first()),
    };
    let Some(cycle) = selected else {
        return Ok(StatsOverview {
            cycle: None,
            cycles: briefs,
            total_count: 0,
            total_amount: money_text(Decimal::ZERO),
            by_class: class_buckets(&[]),
            by_department: Vec::new(),
            by_status: Vec::new(),
        });
    };
    let rows = load_rows(&state.db, cycle.id).await?;
    Ok(StatsOverview {
        cycle: Some(cycle_brief(cycle)),
        cycles: briefs,
        total_count: i64::try_from(rows.len()).unwrap_or(i64::MAX),
        total_amount: money_text(sum_amount(&rows)),
        by_class: class_buckets(&rows),
        by_department: department_buckets(&rows),
        by_status: status_buckets(&rows),
    })
}

pub async fn export_xlsx(
    state: &AppState,
    actor: &CurrentUser,
    query: StatsQuery,
) -> Result<Vec<u8>, AppError> {
    let overview = overview(state, actor, query).await?;
    let mut workbook = Workbook::new();
    let worksheet = workbook.add_worksheet();
    worksheet.set_name("申报统计").map_err(export_error)?;
    let header = Format::new().set_bold();
    for (column, title) in ["维度", "名称", "申报数量", "申请经费（万元）"]
        .into_iter()
        .enumerate()
    {
        worksheet
            .write_string_with_format(0, u16::try_from(column).unwrap_or(0), title, &header)
            .map_err(export_error)?;
    }
    let mut row = 1;
    write_bucket(
        worksheet,
        &mut row,
        "合计",
        overview
            .cycle
            .as_ref()
            .map(|cycle| cycle.title.as_str())
            .unwrap_or("全部批次"),
        overview.total_count,
        &overview.total_amount,
    )?;
    row += 1;
    write_group(worksheet, &mut row, "类别", &overview.by_class)?;
    write_group(worksheet, &mut row, "部门", &overview.by_department)?;
    write_group(worksheet, &mut row, "状态", &overview.by_status)?;
    worksheet.set_column_width(0, 12).map_err(export_error)?;
    worksheet.set_column_width(1, 36).map_err(export_error)?;
    worksheet.set_column_width(2, 14).map_err(export_error)?;
    worksheet.set_column_width(3, 20).map_err(export_error)?;
    workbook.save_to_buffer().map_err(export_error)
}

struct Counted {
    class_code: String,
    status_code: String,
    department_id: String,
    department_name: String,
    amount: Decimal,
}

async fn load_rows(db: &DatabaseConnection, cycle_id: Uuid) -> Result<Vec<Counted>, AppError> {
    let apps = applications::Entity::find()
        .filter(applications::Column::CycleId.eq(cycle_id))
        .find_also_related(departments::Entity)
        .all(db)
        .await?;
    let ids = apps.iter().map(|(app, _)| app.id).collect::<Vec<_>>();
    let budgets = if ids.is_empty() {
        Vec::new()
    } else {
        application_budgets::Entity::find()
            .filter(application_budgets::Column::ApplicationId.is_in(ids))
            .all(db)
            .await?
    };
    let subjects = budget_subjects::Entity::find().all(db).await?;
    let mut lines_by_app: HashMap<Uuid, Vec<application_budgets::Model>> = HashMap::new();
    for line in budgets {
        lines_by_app
            .entry(line.application_id)
            .or_default()
            .push(line);
    }
    Ok(apps
        .into_iter()
        .map(|(app, department)| {
            let family = family_of(&app.project_class);
            let matched = subjects
                .iter()
                .filter(|subject| subject.applies_to == family)
                .cloned()
                .collect::<Vec<_>>();
            let lines = lines_by_app.remove(&app.id).unwrap_or_default();
            Counted {
                class_code: class_code(&app.project_class).to_owned(),
                status_code: status_code(&app.status).to_owned(),
                department_id: department
                    .as_ref()
                    .map(|item| item.id.to_string())
                    .unwrap_or_default(),
                department_name: department
                    .map(|item| item.name)
                    .unwrap_or_else(|| "未分配部门".to_owned()),
                amount: budget_total(&matched, &lines),
            }
        })
        .collect())
}

fn class_buckets(rows: &[Counted]) -> Vec<StatsBucket> {
    ["social", "natural", "dual_high"]
        .into_iter()
        .map(|code| {
            bucket(
                code,
                class_label(code),
                rows.iter().filter(|row| row.class_code == code),
            )
        })
        .collect()
}

fn department_buckets(rows: &[Counted]) -> Vec<StatsBucket> {
    let mut grouped: HashMap<&str, Vec<&Counted>> = HashMap::new();
    for row in rows {
        grouped
            .entry(row.department_id.as_str())
            .or_default()
            .push(row);
    }
    let mut buckets = grouped
        .into_iter()
        .map(|(code, items)| {
            let label = items
                .first()
                .map(|item| item.department_name.clone())
                .unwrap_or_else(|| "未分配部门".to_owned());
            bucket(code, &label, items.iter().copied())
        })
        .collect::<Vec<_>>();
    buckets.sort_by(|left, right| {
        right
            .count
            .cmp(&left.count)
            .then_with(|| left.label.cmp(&right.label))
    });
    buckets
}

fn status_buckets(rows: &[Counted]) -> Vec<StatsBucket> {
    [
        ApplicationStatus::Draft,
        ApplicationStatus::Submitted,
        ApplicationStatus::DeptReturned,
        ApplicationStatus::DeptRejected,
        ApplicationStatus::DeptApproved,
        ApplicationStatus::OfficeReturned,
        ApplicationStatus::OfficeRejected,
        ApplicationStatus::OfficeApproved,
        ApplicationStatus::ExpertReviewing,
        ApplicationStatus::ExpertRejected,
        ApplicationStatus::Established,
    ]
    .into_iter()
    .filter_map(|status| {
        let code = status_code(&status);
        let matched = rows.iter().filter(|row| row.status_code == code);
        let count = matched.clone().count();
        (count > 0).then(|| bucket(code, status_label(code), matched))
    })
    .collect()
}

fn bucket<'a>(
    code: &str,
    label: &str,
    rows: impl Iterator<Item = &'a Counted> + Clone,
) -> StatsBucket {
    let count = rows.clone().count();
    let amount = rows.fold(Decimal::ZERO, |total, row| total + row.amount);
    StatsBucket {
        code: code.to_owned(),
        label: label.to_owned(),
        count: i64::try_from(count).unwrap_or(i64::MAX),
        amount: money_text(amount),
    }
}

fn sum_amount(rows: &[Counted]) -> Decimal {
    rows.iter()
        .fold(Decimal::ZERO, |total, row| total + row.amount)
}

fn cycle_brief(cycle: &application_cycles::Model) -> CycleBrief {
    CycleBrief {
        id: cycle.id,
        cycle_year: cycle.cycle_year,
        title: cycle.title.clone(),
        status: match cycle.status {
            entity::sea_orm_active_enums::CycleStatus::Open => "open",
            entity::sea_orm_active_enums::CycleStatus::Closed => "closed",
            entity::sea_orm_active_enums::CycleStatus::Draft => "draft",
        }
        .to_owned(),
    }
}

fn class_label(code: &str) -> &'static str {
    match code {
        "social" => "社会科学",
        "natural" => "自然科学",
        "dual_high" => "双高专项",
        _ => "其他",
    }
}

fn status_label(code: &str) -> &'static str {
    match code {
        "draft" => "草稿",
        "submitted" => "待系审核",
        "dept_returned" => "系部退回",
        "dept_rejected" => "系部不通过",
        "dept_approved" => "待科研处审核",
        "office_returned" => "科研处退回",
        "office_rejected" => "科研处不通过",
        "office_approved" => "待指派专家",
        "expert_reviewing" => "专家盲审",
        "expert_rejected" => "不予立项",
        "established" => "拟立项",
        _ => "其他",
    }
}

fn write_group(
    worksheet: &mut rust_xlsxwriter::Worksheet,
    row: &mut u32,
    dimension: &str,
    buckets: &[StatsBucket],
) -> Result<(), AppError> {
    if buckets.is_empty() {
        write_bucket(worksheet, row, dimension, "无", 0, "0.00")?;
        *row += 1;
        return Ok(());
    }
    for bucket in buckets {
        write_bucket(
            worksheet,
            row,
            dimension,
            &bucket.label,
            bucket.count,
            &bucket.amount,
        )?;
        *row += 1;
    }
    *row += 1;
    Ok(())
}

fn write_bucket(
    worksheet: &mut rust_xlsxwriter::Worksheet,
    row: &mut u32,
    dimension: &str,
    name: &str,
    count: i64,
    amount: &str,
) -> Result<(), AppError> {
    worksheet
        .write_string(*row, 0, dimension)
        .map_err(export_error)?;
    worksheet
        .write_string(*row, 1, name)
        .map_err(export_error)?;
    worksheet
        .write_number(*row, 2, count as f64)
        .map_err(export_error)?;
    worksheet
        .write_string(*row, 3, amount)
        .map_err(export_error)?;
    Ok(())
}

fn export_error(error: impl std::fmt::Display) -> AppError {
    tracing::error!(error = %error, "导出统计表失败");
    AppError::Internal("导出统计表失败".to_owned())
}

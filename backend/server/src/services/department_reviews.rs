//! 系审核：本部门待审列表、本人已审列表、全文查看和一次结论。
//!
//! 结论、状态和审计写在同一个事务里。状态更新带 `submitted` 条件，
//! 影响 0 行就回滚，避免两个人同时审同一份申报书。

use std::collections::HashMap;

use entity::application_budgets;
use entity::applications;
use entity::budget_subjects;
use entity::departments;
use entity::reviews;
use entity::sea_orm_active_enums::{
    ApplicationStatus, BudgetFamily, ReviewDecision, ReviewStage, UserRole,
};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder, Set,
    TransactionTrait,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use uuid::Uuid;

use crate::auth::{CurrentUser, require_role};
use crate::error::{AppError, FieldError};
use crate::services::{applications as forms, workflow};
use crate::support::audit::{AuditEntry, record};
use crate::support::time::now;

#[derive(Debug, Deserialize)]
pub struct DepartmentDecisionInput {
    pub decision: String,
    pub opinion: Option<String>,
    pub facts_accurate: bool,
    pub team_qualified: bool,
    pub resources_ready: bool,
    pub agree_to_manage: bool,
}

#[derive(Debug, Serialize)]
pub struct ReviewListItem {
    pub id: Uuid,
    pub title: Option<String>,
    pub project_class: String,
    pub principal_name: Option<String>,
    pub professional_title: Option<String>,
    pub department_name: String,
    pub submitted_at: Option<String>,
    pub status: String,
    pub budget_total: String,
    pub decision: Option<String>,
    pub opinion: Option<String>,
    pub reviewed_at: Option<String>,
    pub facts_accurate: Option<bool>,
    pub team_qualified: Option<bool>,
    pub resources_ready: Option<bool>,
    pub agree_to_manage: Option<bool>,
}

pub async fn queue(
    db: &DatabaseConnection,
    user: &CurrentUser,
    scope: &str,
) -> Result<Vec<ReviewListItem>, AppError> {
    require_role(user, UserRole::DepartmentReviewer)?;
    match scope {
        "pending" => pending(db, user).await,
        "reviewed" => reviewed(db, user).await,
        _ => Err(AppError::BadRequest("审核列表范围无效".to_owned())),
    }
}

pub async fn open(
    db: &DatabaseConnection,
    user: &CurrentUser,
    id: Uuid,
) -> Result<forms::ApplicationView, AppError> {
    let app = load_application(db, id).await?;
    ensure_scope(user, app.department_id)?;
    if app.status == ApplicationStatus::Submitted {
        align_submitted_reviews(db, std::slice::from_ref(&app)).await?;
    }
    let app = load_application(db, id).await?;
    if app.status != ApplicationStatus::Submitted {
        let mine = reviews::Entity::find()
            .filter(reviews::Column::ApplicationId.eq(id))
            .filter(reviews::Column::ReviewerId.eq(user.id))
            .filter(reviews::Column::Stage.eq(ReviewStage::Department))
            .one(db)
            .await?;
        if mine.is_none() {
            return Err(AppError::Conflict(
                "这份申报书当前不在系审核环节".to_owned(),
            ));
        }
    }
    forms::load_view(db, id).await
}

pub async fn decide(
    db: &DatabaseConnection,
    user: &CurrentUser,
    id: Uuid,
    input: DepartmentDecisionInput,
) -> Result<ReviewListItem, AppError> {
    let (decision, opinion) = check_decision(&input)?;
    let app = load_application(db, id).await?;
    ensure_scope(user, app.department_id)?;
    if app.status != ApplicationStatus::Submitted {
        return Err(AppError::Conflict(
            "只有待系审核的申报书可以审核".to_owned(),
        ));
    }

    let txn = db.begin().await?;
    let saved_review =
        match write_decision(&txn, user, id, decision.clone(), &opinion, &input).await {
            Ok(review) => review,
            Err(error) => {
                if let Err(rollback_error) = txn.rollback().await {
                    tracing::error!(error = %rollback_error, "系审核事务回滚失败");
                }
                return Err(error);
            }
        };
    txn.commit().await?;

    let mut saved = load_application(db, id).await?;
    if saved.status == ApplicationStatus::Submitted {
        match workflow::transit(
            db,
            id,
            ApplicationStatus::Submitted,
            target_status(decision),
        )
        .await
        {
            Ok(()) => saved = load_application(db, id).await?,
            Err(error) => {
                tracing::error!(error = %error, "系审核结论已写入，申报状态没有更新");
                return Err(AppError::Conflict(
                    "系审核结论已保存，但申报状态仍是待系审核，请刷新后重试".to_owned(),
                ));
            }
        }
    }
    if saved.status == ApplicationStatus::Submitted {
        return Err(AppError::Conflict(
            "系审核结论已保存，但申报状态仍是待系审核，请刷新后重试".to_owned(),
        ));
    }
    let names = department_names(db).await?;
    let totals = budget_totals(db, std::slice::from_ref(&saved)).await?;
    Ok(to_item(&saved, &names, &totals, Some(&saved_review)))
}

async fn write_decision<C>(
    txn: &C,
    user: &CurrentUser,
    id: Uuid,
    decision: ReviewDecision,
    opinion: &Option<String>,
    input: &DepartmentDecisionInput,
) -> Result<reviews::Model, AppError>
where
    C: sea_orm::ConnectionTrait,
{
    let current = applications::Entity::find_by_id(id)
        .one(txn)
        .await?
        .ok_or_else(|| AppError::NotFound("申报书不存在".to_owned()))?;
    if current.status != ApplicationStatus::Submitted {
        return Err(AppError::Conflict(
            "申报书状态已变化，请刷新后重试".to_owned(),
        ));
    }
    let existing = reviews::Entity::find()
        .filter(reviews::Column::ApplicationId.eq(id))
        .filter(reviews::Column::Stage.eq(ReviewStage::Department))
        .one(txn)
        .await?;
    if existing.is_some() {
        return Err(AppError::Conflict("这份申报书已经完成系审核".to_owned()));
    }

    let target = target_status(decision.clone());
    workflow::transit(txn, id, ApplicationStatus::Submitted, target.clone()).await?;
    let timestamp = now();
    let saved_review = reviews::ActiveModel {
        id: Set(Uuid::new_v4()),
        application_id: Set(id),
        stage: Set(ReviewStage::Department),
        reviewer_id: Set(user.id),
        assignment_id: Set(None),
        decision: Set(decision.clone()),
        opinion: Set(opinion.clone()),
        score: Set(None),
        facts_accurate: Set(Some(input.facts_accurate)),
        team_qualified: Set(Some(input.team_qualified)),
        resources_ready: Set(Some(input.resources_ready)),
        agree_to_manage: Set(Some(input.agree_to_manage)),
        is_blind: Set(false),
        reviewed_at: Set(Some(timestamp)),
        created_at: Set(timestamp),
        updated_at: Set(timestamp),
    }
    .insert(txn)
    .await?;
    record(
        txn,
        AuditEntry {
            actor_id: Some(user.id),
            action: "department_review",
            target_type: "application",
            target_id: Some(id),
            detail: Some(json!({
                "from": "submitted",
                "to": forms::status_code(&target),
                "decision": decision_code(&decision),
                "opinion": opinion,
            })),
            ip_address: None,
            user_agent: None,
        },
    )
    .await?;
    Ok(saved_review)
}

async fn pending(
    db: &DatabaseConnection,
    user: &CurrentUser,
) -> Result<Vec<ReviewListItem>, AppError> {
    let mut query = applications::Entity::find()
        .filter(applications::Column::Status.eq(ApplicationStatus::Submitted));
    if !user.has_role(UserRole::Admin) {
        let department_id = user
            .department_id
            .ok_or_else(|| AppError::Forbidden("请先为账号分配部门".to_owned()))?;
        query = query.filter(applications::Column::DepartmentId.eq(department_id));
    }
    let apps = query
        .clone()
        .order_by_desc(applications::Column::SubmittedAt)
        .all(db)
        .await?;
    align_submitted_reviews(db, &apps).await?;
    let apps = query
        .order_by_desc(applications::Column::SubmittedAt)
        .all(db)
        .await?;
    pack(db, &apps, &[]).await
}

/// 系审核结论已经写下、申报状态却仍是待系审核时，把状态补到和结论一致。
async fn align_submitted_reviews(
    db: &DatabaseConnection,
    apps: &[applications::Model],
) -> Result<(), AppError> {
    let ids: Vec<Uuid> = apps
        .iter()
        .filter(|app| app.status == ApplicationStatus::Submitted)
        .map(|app| app.id)
        .collect();
    if ids.is_empty() {
        return Ok(());
    }
    let rows = reviews::Entity::find()
        .filter(reviews::Column::ApplicationId.is_in(ids))
        .filter(reviews::Column::Stage.eq(ReviewStage::Department))
        .filter(reviews::Column::Decision.ne(ReviewDecision::Pending))
        .all(db)
        .await?;
    for review in rows {
        let target = target_status(review.decision.clone());
        if target == ApplicationStatus::Submitted {
            continue;
        }
        match workflow::transit(
            db,
            review.application_id,
            ApplicationStatus::Submitted,
            target,
        )
        .await
        {
            Ok(()) => {}
            Err(AppError::Conflict(_)) => {}
            Err(error) => return Err(error),
        }
    }
    Ok(())
}

async fn reviewed(
    db: &DatabaseConnection,
    user: &CurrentUser,
) -> Result<Vec<ReviewListItem>, AppError> {
    let rows = reviews::Entity::find()
        .filter(reviews::Column::ReviewerId.eq(user.id))
        .filter(reviews::Column::Stage.eq(ReviewStage::Department))
        .filter(reviews::Column::Decision.ne(ReviewDecision::Pending))
        .order_by_desc(reviews::Column::ReviewedAt)
        .all(db)
        .await?;
    if rows.is_empty() {
        return Ok(Vec::new());
    }
    let ids: Vec<Uuid> = rows.iter().map(|row| row.application_id).collect();
    let apps = applications::Entity::find()
        .filter(applications::Column::Id.is_in(ids))
        .all(db)
        .await?;
    pack(db, &apps, &rows).await
}

async fn pack(
    db: &DatabaseConnection,
    apps: &[applications::Model],
    reviews_in_order: &[reviews::Model],
) -> Result<Vec<ReviewListItem>, AppError> {
    let names = department_names(db).await?;
    let totals = budget_totals(db, apps).await?;
    if reviews_in_order.is_empty() {
        return Ok(apps
            .iter()
            .map(|app| to_item(app, &names, &totals, None))
            .collect());
    }
    let by_id: HashMap<Uuid, &applications::Model> = apps.iter().map(|app| (app.id, app)).collect();
    Ok(reviews_in_order
        .iter()
        .filter_map(|review| {
            by_id
                .get(&review.application_id)
                .map(|app| to_item(app, &names, &totals, Some(review)))
        })
        .collect())
}

fn to_item(
    app: &applications::Model,
    names: &HashMap<Uuid, String>,
    totals: &HashMap<Uuid, String>,
    review: Option<&reviews::Model>,
) -> ReviewListItem {
    ReviewListItem {
        id: app.id,
        title: app.title.clone(),
        project_class: forms::class_code(&app.project_class).to_owned(),
        principal_name: app.principal_name.clone(),
        professional_title: app.professional_title.clone(),
        department_name: app
            .department_id
            .and_then(|id| names.get(&id).cloned())
            .unwrap_or_else(|| "未分配部门".to_owned()),
        submitted_at: app.submitted_at.map(|value| value.to_rfc3339()),
        status: forms::status_code(&app.status).to_owned(),
        budget_total: totals
            .get(&app.id)
            .cloned()
            .unwrap_or_else(|| "0.00".to_owned()),
        decision: review.map(|row| decision_code(&row.decision).to_owned()),
        opinion: review.and_then(|row| row.opinion.clone()),
        reviewed_at: review.and_then(|row| row.reviewed_at.map(|value| value.to_rfc3339())),
        facts_accurate: review.and_then(|row| row.facts_accurate),
        team_qualified: review.and_then(|row| row.team_qualified),
        resources_ready: review.and_then(|row| row.resources_ready),
        agree_to_manage: review.and_then(|row| row.agree_to_manage),
    }
}

fn check_decision(
    input: &DepartmentDecisionInput,
) -> Result<(ReviewDecision, Option<String>), AppError> {
    let decision = match input.decision.as_str() {
        "approved" => ReviewDecision::Approved,
        "returned" => ReviewDecision::Returned,
        "rejected" => ReviewDecision::Rejected,
        _ => {
            return Err(invalid("decision", "审核结论只能是通过、退回或不通过"));
        }
    };
    let opinion = input
        .opinion
        .as_deref()
        .map(str::trim)
        .filter(|text| !text.is_empty())
        .map(str::to_owned);
    if matches!(
        decision,
        ReviewDecision::Returned | ReviewDecision::Rejected
    ) && opinion.is_none()
    {
        return Err(invalid("opinion", "退回或不通过必须填写意见"));
    }
    let confirmed = input.facts_accurate
        && input.team_qualified
        && input.resources_ready
        && input.agree_to_manage;
    if decision == ReviewDecision::Approved && !confirmed {
        return Err(invalid("checks", "通过时四项确认都要勾选"));
    }
    Ok((decision, opinion))
}

fn target_status(decision: ReviewDecision) -> ApplicationStatus {
    match decision {
        ReviewDecision::Approved => ApplicationStatus::DeptApproved,
        ReviewDecision::Returned => ApplicationStatus::DeptReturned,
        ReviewDecision::Rejected => ApplicationStatus::DeptRejected,
        ReviewDecision::Pending => ApplicationStatus::Submitted,
    }
}

fn decision_code(decision: &ReviewDecision) -> &'static str {
    match decision {
        ReviewDecision::Pending => "pending",
        ReviewDecision::Approved => "approved",
        ReviewDecision::Rejected => "rejected",
        ReviewDecision::Returned => "returned",
    }
}

fn ensure_scope(user: &CurrentUser, department_id: Option<Uuid>) -> Result<(), AppError> {
    require_role(user, UserRole::DepartmentReviewer)?;
    if user.has_role(UserRole::Admin) {
        return Ok(());
    }
    let own = user
        .department_id
        .ok_or_else(|| AppError::Forbidden("请先为账号分配部门".to_owned()))?;
    if department_id == Some(own) {
        Ok(())
    } else {
        Err(AppError::Forbidden("只能审核本部门的申报书".to_owned()))
    }
}

async fn load_application(
    db: &DatabaseConnection,
    id: Uuid,
) -> Result<applications::Model, AppError> {
    applications::Entity::find_by_id(id)
        .one(db)
        .await?
        .ok_or_else(|| AppError::NotFound("申报书不存在".to_owned()))
}

async fn department_names(db: &DatabaseConnection) -> Result<HashMap<Uuid, String>, AppError> {
    let rows = departments::Entity::find().all(db).await?;
    Ok(rows.into_iter().map(|row| (row.id, row.name)).collect())
}

async fn budget_totals(
    db: &DatabaseConnection,
    apps: &[applications::Model],
) -> Result<HashMap<Uuid, String>, AppError> {
    let mut totals = HashMap::new();
    if apps.is_empty() {
        return Ok(totals);
    }
    let ids: Vec<Uuid> = apps.iter().map(|app| app.id).collect();
    let lines = application_budgets::Entity::find()
        .filter(application_budgets::Column::ApplicationId.is_in(ids))
        .all(db)
        .await?;
    let subjects = budget_subjects::Entity::find().all(db).await?;
    let social = subjects_for(&subjects, BudgetFamily::Social);
    let natural = subjects_for(&subjects, BudgetFamily::Natural);
    let mut grouped: HashMap<Uuid, Vec<application_budgets::Model>> = HashMap::new();
    for line in lines {
        grouped.entry(line.application_id).or_default().push(line);
    }
    for app in apps {
        let applicable = match forms::family_of(&app.project_class) {
            BudgetFamily::Social => &social,
            BudgetFamily::Natural => &natural,
        };
        let rows = grouped.get(&app.id).map(Vec::as_slice).unwrap_or(&[]);
        totals.insert(
            app.id,
            forms::money_text(forms::budget_total(applicable, rows)),
        );
    }
    Ok(totals)
}

fn subjects_for(
    subjects: &[budget_subjects::Model],
    family: BudgetFamily,
) -> Vec<budget_subjects::Model> {
    subjects
        .iter()
        .filter(|subject| subject.applies_to == family)
        .cloned()
        .collect()
}

fn invalid(field: &str, message: &str) -> AppError {
    AppError::Validation(vec![FieldError {
        field: field.to_owned(),
        message: message.to_owned(),
    }])
}

#[cfg(test)]
mod tests {
    use super::{DepartmentDecisionInput, check_decision};
    use crate::error::AppError;
    use entity::sea_orm_active_enums::ReviewDecision;

    fn input(decision: &str, opinion: Option<&str>, ready: bool) -> DepartmentDecisionInput {
        DepartmentDecisionInput {
            decision: decision.to_owned(),
            opinion: opinion.map(str::to_owned),
            facts_accurate: ready,
            team_qualified: ready,
            resources_ready: ready,
            agree_to_manage: ready,
        }
    }

    fn message(error: AppError) -> String {
        match error {
            AppError::Validation(fields) => fields
                .into_iter()
                .map(|field| field.message)
                .collect::<Vec<_>>()
                .join("；"),
            other => other.to_string(),
        }
    }

    #[test]
    fn approve_requires_all_four_confirmations() {
        let mut body = input("approved", None, true);
        body.resources_ready = false;
        let error = check_decision(&body).expect_err("缺一项确认不能通过");
        assert_eq!(message(error), "通过时四项确认都要勾选");
    }

    #[test]
    fn return_requires_opinion() {
        let error = check_decision(&input("returned", Some("  "), true)).expect_err("空意见");
        assert_eq!(message(error), "退回或不通过必须填写意见");
    }

    #[test]
    fn reject_can_leave_a_confirmation_unchecked() {
        let mut body = input("rejected", Some("研究条件不具备"), true);
        body.team_qualified = false;
        let (decision, opinion) = check_decision(&body).expect("不通过应接受");
        assert_eq!(decision, ReviewDecision::Rejected);
        assert_eq!(opinion.as_deref(), Some("研究条件不具备"));
    }

    #[test]
    fn approve_keeps_an_optional_opinion() {
        let (decision, opinion) =
            check_decision(&input("approved", Some("同意推荐"), true)).expect("通过应接受");
        assert_eq!(decision, ReviewDecision::Approved);
        assert_eq!(opinion.as_deref(), Some("同意推荐"));
    }
}

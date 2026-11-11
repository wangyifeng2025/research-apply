//! 科研处审核、专家指派和立项结论。
//!
//! 科研处看到全院。通过时写入封面编号；指派避开负责人、参加者和推荐人；
//! 全部专家提交后才能立项或不予立项。

use std::collections::{HashMap, HashSet};

use entity::application_budgets;
use entity::application_members;
use entity::applications;
use entity::budget_subjects;
use entity::departments;
use entity::expert_assignments;
use entity::recommenders;
use entity::reviews;
use entity::sea_orm_active_enums::{
    ApplicationStatus, BudgetFamily, ReviewDecision, ReviewStage, UserRole,
};
use entity::user_roles;
use entity::users;
use sea_orm::sea_query::Expr;
use sea_orm::{
    ActiveEnum, ActiveModelTrait, ColumnTrait, ConnectionTrait, DatabaseConnection, EntityTrait,
    QueryFilter, QueryOrder, Set, TransactionTrait,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use uuid::Uuid;

use crate::auth::{CurrentUser, require_role};
use crate::error::{AppError, FieldError};
use crate::services::applications::ApplicationView;
use crate::services::{applications as forms, workflow};
use crate::support::audit::{AuditEntry, record};
use crate::support::time::now;

#[derive(Debug, Deserialize)]
pub struct OfficeDecisionInput {
    pub decision: String,
    pub opinion: Option<String>,
    pub code: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct AssignInput {
    pub expert_ids: Vec<Uuid>,
}

#[derive(Debug, Deserialize)]
pub struct ConclusionInput {
    pub decision: String,
    pub opinion: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct OfficeListItem {
    pub id: Uuid,
    pub title: Option<String>,
    pub project_class: String,
    pub principal_name: Option<String>,
    pub professional_title: Option<String>,
    pub department_name: String,
    pub submitted_at: Option<String>,
    pub status: String,
    pub budget_total: String,
    pub code: Option<String>,
    pub assigned_count: usize,
    pub reviewed_count: usize,
    pub average_score: Option<String>,
    pub decision: Option<String>,
    pub opinion: Option<String>,
    pub reviewed_at: Option<String>,
    pub conclusion_opinion: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct AssignmentView {
    pub id: Uuid,
    pub expert_id: Uuid,
    pub expert_name: String,
    pub research_expertise: Option<String>,
    pub reviewed: bool,
    pub decision: Option<String>,
    pub opinion: Option<String>,
    pub score: Option<String>,
    pub reviewed_at: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct OfficeDetail {
    pub application: ApplicationView,
    pub code: Option<String>,
    pub conclusion_opinion: Option<String>,
    pub assignments: Vec<AssignmentView>,
    pub average_score: Option<String>,
    pub ready_to_conclude: bool,
}

#[derive(Debug, Serialize)]
pub struct ExpertOption {
    pub id: Uuid,
    pub real_name: String,
    pub research_expertise: Option<String>,
    pub department_name: String,
}

pub async fn queue(
    db: &DatabaseConnection,
    user: &CurrentUser,
    scope: &str,
) -> Result<Vec<OfficeListItem>, AppError> {
    require_role(user, UserRole::ResearchOffice)?;
    let (apps, own_reviews) = match scope {
        "pending" => (by_status(db, ApplicationStatus::DeptApproved).await?, None),
        "assign" => (
            by_status(db, ApplicationStatus::OfficeApproved).await?,
            None,
        ),
        "reviewing" | "conclude" => {
            let apps = by_status(db, ApplicationStatus::ExpertReviewing).await?;
            (apps, None)
        }
        "established" => (by_status(db, ApplicationStatus::Established).await?, None),
        "expert_rejected" => (
            by_status(db, ApplicationStatus::ExpertRejected).await?,
            None,
        ),
        "returned" | "rejected" => {
            let decision = if scope == "returned" {
                ReviewDecision::Returned
            } else {
                ReviewDecision::Rejected
            };
            let rows = own_office_reviews(db, user.id, decision).await?;
            let ids: Vec<Uuid> = rows.iter().map(|row| row.application_id).collect();
            let apps = if ids.is_empty() {
                Vec::new()
            } else {
                applications::Entity::find()
                    .filter(applications::Column::Id.is_in(ids))
                    .all(db)
                    .await?
            };
            (apps, Some(rows))
        }
        _ => return Err(AppError::BadRequest("审核列表范围无效".to_owned())),
    };
    let mut items = pack(db, &apps, own_reviews.as_deref()).await?;
    if scope == "reviewing" {
        items.retain(|item| item.reviewed_count < item.assigned_count);
    }
    if scope == "conclude" {
        items.retain(|item| item.assigned_count > 0 && item.reviewed_count == item.assigned_count);
    }
    Ok(items)
}

pub async fn open(
    db: &DatabaseConnection,
    user: &CurrentUser,
    id: Uuid,
) -> Result<OfficeDetail, AppError> {
    require_role(user, UserRole::ResearchOffice)?;
    let app = load_application(db, id).await?;
    if !office_visible(&app.status) {
        return Err(AppError::Conflict(
            "这份申报书当前不在科研处环节".to_owned(),
        ));
    }
    let application = forms::load_view(db, id).await?;
    let assignments = assignment_views(db, id).await?;
    let reviewed = assignments.iter().filter(|item| item.reviewed).count();
    let average = average_of(assignments.iter().filter_map(|item| item.score.as_deref()));
    let ready = app.status == ApplicationStatus::ExpertReviewing
        && !assignments.is_empty()
        && reviewed == assignments.len();
    Ok(OfficeDetail {
        application,
        code: app.code,
        conclusion_opinion: app.conclusion_opinion,
        assignments,
        average_score: average,
        ready_to_conclude: ready,
    })
}

pub async fn decide(
    db: &DatabaseConnection,
    user: &CurrentUser,
    id: Uuid,
    input: OfficeDecisionInput,
) -> Result<OfficeListItem, AppError> {
    require_role(user, UserRole::ResearchOffice)?;
    let (decision, opinion, code) = check_office_decision(&input)?;
    let app = load_application(db, id).await?;
    if app.status != ApplicationStatus::DeptApproved {
        return Err(AppError::Conflict(
            "只有待科研处审核的申报书可以审核".to_owned(),
        ));
    }
    let txn = db.begin().await?;
    let saved = match write_office_decision(&txn, user, id, decision.clone(), &opinion, code).await
    {
        Ok(review) => review,
        Err(error) => {
            if let Err(rollback_error) = txn.rollback().await {
                tracing::error!(error = %rollback_error, "科研处审核事务回滚失败");
            }
            return Err(error);
        }
    };
    txn.commit().await?;
    let saved_app = load_application(db, id).await?;
    let names = department_names(db).await?;
    let totals = budget_totals(db, std::slice::from_ref(&saved_app)).await?;
    Ok(to_item(
        &saved_app,
        &names,
        &totals,
        &HashMap::new(),
        Some(&saved),
    ))
}

pub async fn experts(
    db: &DatabaseConnection,
    user: &CurrentUser,
    query: Option<&str>,
) -> Result<Vec<ExpertOption>, AppError> {
    require_role(user, UserRole::ResearchOffice)?;
    let roles = user_roles::Entity::find()
        .filter(user_roles::Column::Role.eq(UserRole::Expert))
        .all(db)
        .await?;
    let ids: Vec<Uuid> = roles.into_iter().map(|row| row.user_id).collect();
    if ids.is_empty() {
        return Ok(Vec::new());
    }
    let rows = users::Entity::find()
        .filter(users::Column::Id.is_in(ids))
        .filter(users::Column::IsActive.eq(true))
        .order_by_asc(users::Column::RealName)
        .all(db)
        .await?;
    let names = department_names(db).await?;
    let needle = query.unwrap_or("").trim();
    Ok(rows
        .into_iter()
        .filter(|row| {
            needle.is_empty()
                || row.real_name.contains(needle)
                || row
                    .research_expertise
                    .as_deref()
                    .is_some_and(|text| text.contains(needle))
        })
        .map(|row| ExpertOption {
            department_name: row
                .department_id
                .and_then(|id| names.get(&id).cloned())
                .unwrap_or_else(|| "未分配部门".to_owned()),
            id: row.id,
            real_name: row.real_name,
            research_expertise: row.research_expertise,
        })
        .collect())
}

pub async fn assign(
    db: &DatabaseConnection,
    user: &CurrentUser,
    id: Uuid,
    input: AssignInput,
) -> Result<OfficeDetail, AppError> {
    require_role(user, UserRole::ResearchOffice)?;
    if input.expert_ids.is_empty() {
        return Err(invalid("expert_ids", "请选择至少一位专家"));
    }
    if input.expert_ids.len() != input.expert_ids.iter().collect::<HashSet<_>>().len() {
        return Err(invalid("expert_ids", "同一位专家不能重复选择"));
    }
    let app = load_application(db, id).await?;
    if !matches!(
        app.status,
        ApplicationStatus::OfficeApproved | ApplicationStatus::ExpertReviewing
    ) {
        return Err(AppError::Conflict(
            "只有科研处已通过或盲审中的申报书可以指派专家".to_owned(),
        ));
    }
    let txn = db.begin().await?;
    if let Err(error) = write_assignments(&txn, user, id, &input.expert_ids).await {
        if let Err(rollback_error) = txn.rollback().await {
            tracing::error!(error = %rollback_error, "指派专家事务回滚失败");
        }
        return Err(error);
    }
    txn.commit().await?;
    open(db, user, id).await
}

pub async fn revoke(
    db: &DatabaseConnection,
    user: &CurrentUser,
    id: Uuid,
    assignment_id: Uuid,
) -> Result<OfficeDetail, AppError> {
    require_role(user, UserRole::ResearchOffice)?;
    let txn = db.begin().await?;
    if let Err(error) = write_revoke(&txn, user, id, assignment_id).await {
        if let Err(rollback_error) = txn.rollback().await {
            tracing::error!(error = %rollback_error, "撤销指派事务回滚失败");
        }
        return Err(error);
    }
    txn.commit().await?;
    open(db, user, id).await
}

pub async fn conclude(
    db: &DatabaseConnection,
    user: &CurrentUser,
    id: Uuid,
    input: ConclusionInput,
) -> Result<OfficeListItem, AppError> {
    require_role(user, UserRole::ResearchOffice)?;
    let (status, opinion) = check_conclusion(&input)?;
    let txn = db.begin().await?;
    if let Err(error) = write_conclusion(&txn, user, id, status.clone(), &opinion).await {
        if let Err(rollback_error) = txn.rollback().await {
            tracing::error!(error = %rollback_error, "立项结论事务回滚失败");
        }
        return Err(error);
    }
    txn.commit().await?;
    let saved = load_application(db, id).await?;
    let names = department_names(db).await?;
    let totals = budget_totals(db, std::slice::from_ref(&saved)).await?;
    let progress = progress_for(db, &[saved.id]).await?;
    Ok(to_item(&saved, &names, &totals, &progress, None))
}

async fn write_office_decision<C>(
    txn: &C,
    user: &CurrentUser,
    id: Uuid,
    decision: ReviewDecision,
    opinion: &Option<String>,
    code: Option<String>,
) -> Result<reviews::Model, AppError>
where
    C: ConnectionTrait,
{
    let current = load_in(txn, id).await?;
    if current.status != ApplicationStatus::DeptApproved {
        return Err(AppError::Conflict(
            "申报书状态已变化，请刷新后重试".to_owned(),
        ));
    }
    let existing = reviews::Entity::find()
        .filter(reviews::Column::ApplicationId.eq(id))
        .filter(reviews::Column::Stage.eq(ReviewStage::ResearchOffice))
        .one(txn)
        .await?;
    if existing.is_some() {
        return Err(AppError::Conflict(
            "这份申报书已经完成科研处审核".to_owned(),
        ));
    }
    if let Some(code) = &code {
        let taken = applications::Entity::find()
            .filter(applications::Column::Code.eq(code.clone()))
            .one(txn)
            .await?;
        if taken.is_some() {
            return Err(AppError::Conflict("封面编号已被占用".to_owned()));
        }
    }
    let target = office_target(decision.clone());
    workflow::transit(txn, id, ApplicationStatus::DeptApproved, target.clone()).await?;
    if let Some(code) = code.clone() {
        applications::Entity::update_many()
            .col_expr(applications::Column::Code, Expr::value(Some(code)))
            .col_expr(applications::Column::UpdatedAt, Expr::value(now()))
            .filter(applications::Column::Id.eq(id))
            .exec(txn)
            .await?;
    }
    let timestamp = now();
    let saved = reviews::ActiveModel {
        id: Set(Uuid::new_v4()),
        application_id: Set(id),
        stage: Set(ReviewStage::ResearchOffice),
        reviewer_id: Set(user.id),
        assignment_id: Set(None),
        decision: Set(decision.clone()),
        opinion: Set(opinion.clone()),
        score: Set(None),
        facts_accurate: Set(None),
        team_qualified: Set(None),
        resources_ready: Set(None),
        agree_to_manage: Set(None),
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
            action: "office_review",
            target_type: "application",
            target_id: Some(id),
            detail: Some(json!({
                "from": "dept_approved",
                "to": forms::status_code(&target),
                "decision": decision_code(&decision),
                "code": code,
            })),
            ip_address: None,
            user_agent: None,
        },
    )
    .await?;
    Ok(saved)
}

async fn write_assignments<C>(
    txn: &C,
    user: &CurrentUser,
    id: Uuid,
    expert_ids: &[Uuid],
) -> Result<(), AppError>
where
    C: ConnectionTrait,
{
    let current = load_in(txn, id).await?;
    if !matches!(
        current.status,
        ApplicationStatus::OfficeApproved | ApplicationStatus::ExpertReviewing
    ) {
        return Err(AppError::Conflict(
            "申报书状态已变化，请刷新后重试".to_owned(),
        ));
    }
    let members = application_members::Entity::find()
        .filter(application_members::Column::ApplicationId.eq(id))
        .all(txn)
        .await?;
    let recommender = recommenders::Entity::find_by_id(id).one(txn).await?;
    let existing = expert_assignments::Entity::find()
        .filter(expert_assignments::Column::ApplicationId.eq(id))
        .all(txn)
        .await?;
    let member_names: Vec<&str> = members.iter().map(|row| row.name.as_str()).collect();
    let timestamp = now();
    for expert_id in expert_ids {
        if *expert_id == user.id {
            return Err(invalid("expert_ids", "不能把申报书指派给自己"));
        }
        if existing.iter().any(|row| row.expert_id == *expert_id) {
            return Err(AppError::Conflict("同一位专家不能重复指派".to_owned()));
        }
        let expert = users::Entity::find_by_id(*expert_id)
            .one(txn)
            .await?
            .ok_or_else(|| invalid("expert_ids", "专家不存在"))?;
        if !expert.is_active {
            return Err(invalid("expert_ids", "专家账号已停用"));
        }
        let role = user_roles::Entity::find()
            .filter(user_roles::Column::UserId.eq(*expert_id))
            .filter(user_roles::Column::Role.eq(UserRole::Expert))
            .one(txn)
            .await?;
        if role.is_none() {
            return Err(invalid("expert_ids", "只能指派评审专家"));
        }
        if *expert_id == current.principal_id
            || person_conflict(
                &expert.real_name,
                current.principal_name.as_deref().unwrap_or(""),
                &member_names,
                recommender.as_ref().map(|row| row.name.as_str()),
            )
        {
            return Err(invalid("expert_ids", "不能指派负责人、参加者或推荐人"));
        }
        expert_assignments::ActiveModel {
            id: Set(Uuid::new_v4()),
            application_id: Set(id),
            expert_id: Set(*expert_id),
            assigned_by: Set(user.id),
            assigned_at: Set(timestamp),
            created_at: Set(timestamp),
            updated_at: Set(timestamp),
        }
        .insert(txn)
        .await?;
    }
    if current.status == ApplicationStatus::OfficeApproved {
        workflow::transit(
            txn,
            id,
            ApplicationStatus::OfficeApproved,
            ApplicationStatus::ExpertReviewing,
        )
        .await?;
    }
    record(
        txn,
        AuditEntry {
            actor_id: Some(user.id),
            action: "expert_assigned",
            target_type: "application",
            target_id: Some(id),
            detail: Some(json!({ "expert_ids": expert_ids })),
            ip_address: None,
            user_agent: None,
        },
    )
    .await?;
    Ok(())
}

async fn write_revoke<C>(
    txn: &C,
    user: &CurrentUser,
    id: Uuid,
    assignment_id: Uuid,
) -> Result<(), AppError>
where
    C: ConnectionTrait,
{
    let current = load_in(txn, id).await?;
    let assignment = expert_assignments::Entity::find_by_id(assignment_id)
        .one(txn)
        .await?
        .ok_or_else(|| AppError::NotFound("指派不存在".to_owned()))?;
    if assignment.application_id != id {
        return Err(AppError::NotFound("指派不存在".to_owned()));
    }
    let reviewed = reviews::Entity::find()
        .filter(reviews::Column::AssignmentId.eq(assignment_id))
        .one(txn)
        .await?;
    if reviewed.is_some() {
        return Err(AppError::Conflict("专家已经提交评审，不能撤销".to_owned()));
    }
    expert_assignments::Entity::delete_by_id(assignment_id)
        .exec(txn)
        .await?;
    let remaining = expert_assignments::Entity::find()
        .filter(expert_assignments::Column::ApplicationId.eq(id))
        .all(txn)
        .await?;
    if remaining.is_empty() && current.status == ApplicationStatus::ExpertReviewing {
        workflow::transit(
            txn,
            id,
            ApplicationStatus::ExpertReviewing,
            ApplicationStatus::OfficeApproved,
        )
        .await?;
    }
    record(
        txn,
        AuditEntry {
            actor_id: Some(user.id),
            action: "expert_assignment_revoked",
            target_type: "application",
            target_id: Some(id),
            detail: Some(
                json!({ "assignment_id": assignment_id, "expert_id": assignment.expert_id }),
            ),
            ip_address: None,
            user_agent: None,
        },
    )
    .await?;
    Ok(())
}

async fn write_conclusion<C>(
    txn: &C,
    user: &CurrentUser,
    id: Uuid,
    status: ApplicationStatus,
    opinion: &str,
) -> Result<(), AppError>
where
    C: ConnectionTrait,
{
    let current = load_in(txn, id).await?;
    if current.status != ApplicationStatus::ExpertReviewing {
        return Err(AppError::Conflict(
            "只有盲审中的申报书可以做立项结论".to_owned(),
        ));
    }
    let assignments = expert_assignments::Entity::find()
        .filter(expert_assignments::Column::ApplicationId.eq(id))
        .all(txn)
        .await?;
    if assignments.is_empty() {
        return Err(AppError::Conflict("还没有指派专家".to_owned()));
    }
    let reviews = reviews::Entity::find()
        .filter(reviews::Column::ApplicationId.eq(id))
        .filter(reviews::Column::Stage.eq(ReviewStage::Expert))
        .all(txn)
        .await?;
    let reviewed_ids: HashSet<Uuid> = reviews.iter().filter_map(|row| row.assignment_id).collect();
    if assignments
        .iter()
        .any(|row| !reviewed_ids.contains(&row.id))
    {
        return Err(AppError::Conflict("还有专家没有提交评审".to_owned()));
    }
    if !workflow::allows(current.status.clone(), status.clone()) {
        return Err(AppError::Conflict("当前状态不能这样流转".to_owned()));
    }
    let timestamp = now();
    let result = applications::Entity::update_many()
        .col_expr(applications::Column::Status, Expr::value(status.to_value()))
        .col_expr(
            applications::Column::ConcludedBy,
            Expr::value(Some(user.id)),
        )
        .col_expr(
            applications::Column::ConcludedAt,
            Expr::value(Some(timestamp)),
        )
        .col_expr(
            applications::Column::ConclusionOpinion,
            Expr::value(Some(opinion.to_owned())),
        )
        .col_expr(applications::Column::UpdatedAt, Expr::value(timestamp))
        .filter(applications::Column::Id.eq(id))
        .filter(applications::Column::Status.eq(ApplicationStatus::ExpertReviewing))
        .exec(txn)
        .await?;
    if result.rows_affected == 0 {
        return Err(AppError::Conflict(
            "申报书状态已变化，请刷新后重试".to_owned(),
        ));
    }
    record(
        txn,
        AuditEntry {
            actor_id: Some(user.id),
            action: "application_concluded",
            target_type: "application",
            target_id: Some(id),
            detail: Some(json!({
                "to": forms::status_code(&status),
                "opinion": opinion,
            })),
            ip_address: None,
            user_agent: None,
        },
    )
    .await?;
    Ok(())
}

fn check_office_decision(
    input: &OfficeDecisionInput,
) -> Result<(ReviewDecision, Option<String>, Option<String>), AppError> {
    let decision = match input.decision.as_str() {
        "approved" => ReviewDecision::Approved,
        "returned" => ReviewDecision::Returned,
        "rejected" => ReviewDecision::Rejected,
        _ => return Err(invalid("decision", "审核结论只能是通过、退回或不通过")),
    };
    let opinion = blank_to_none(input.opinion.as_deref());
    if matches!(
        decision,
        ReviewDecision::Returned | ReviewDecision::Rejected
    ) && opinion.is_none()
    {
        return Err(invalid("opinion", "退回或不通过必须填写意见"));
    }
    let code = match decision {
        ReviewDecision::Approved => {
            let code = input.code.as_deref().unwrap_or("").trim();
            if code.is_empty() {
                return Err(invalid("code", "通过时必须填写封面编号"));
            }
            if code.chars().count() > 32 {
                return Err(invalid("code", "封面编号不能超过 32 个字符"));
            }
            Some(code.to_owned())
        }
        _ => None,
    };
    Ok((decision, opinion, code))
}

fn check_conclusion(input: &ConclusionInput) -> Result<(ApplicationStatus, String), AppError> {
    let status = match input.decision.as_str() {
        "established" => ApplicationStatus::Established,
        "expert_rejected" => ApplicationStatus::ExpertRejected,
        _ => return Err(invalid("decision", "立项结论只能是立项或不予立项")),
    };
    let opinion = blank_to_none(input.opinion.as_deref())
        .ok_or_else(|| invalid("opinion", "请填写结论说明"))?;
    Ok((status, opinion))
}

fn person_conflict(
    expert_name: &str,
    principal_name: &str,
    member_names: &[&str],
    recommender_name: Option<&str>,
) -> bool {
    same_person(expert_name, principal_name)
        || member_names
            .iter()
            .any(|name| same_person(expert_name, name))
        || recommender_name.is_some_and(|name| same_person(expert_name, name))
}

fn same_person(left: &str, right: &str) -> bool {
    let left = left.trim();
    let right = right.trim();
    !left.is_empty() && left == right
}

fn office_target(decision: ReviewDecision) -> ApplicationStatus {
    match decision {
        ReviewDecision::Approved => ApplicationStatus::OfficeApproved,
        ReviewDecision::Returned => ApplicationStatus::OfficeReturned,
        ReviewDecision::Rejected => ApplicationStatus::OfficeRejected,
        ReviewDecision::Pending => ApplicationStatus::DeptApproved,
    }
}

fn office_visible(status: &ApplicationStatus) -> bool {
    matches!(
        status,
        ApplicationStatus::DeptApproved
            | ApplicationStatus::OfficeApproved
            | ApplicationStatus::OfficeReturned
            | ApplicationStatus::OfficeRejected
            | ApplicationStatus::ExpertReviewing
            | ApplicationStatus::ExpertRejected
            | ApplicationStatus::Established
    )
}

fn decision_code(decision: &ReviewDecision) -> &'static str {
    match decision {
        ReviewDecision::Pending => "pending",
        ReviewDecision::Approved => "approved",
        ReviewDecision::Rejected => "rejected",
        ReviewDecision::Returned => "returned",
    }
}

fn blank_to_none(value: Option<&str>) -> Option<String> {
    value
        .map(str::trim)
        .filter(|text| !text.is_empty())
        .map(str::to_owned)
}

async fn by_status(
    db: &DatabaseConnection,
    status: ApplicationStatus,
) -> Result<Vec<applications::Model>, AppError> {
    Ok(applications::Entity::find()
        .filter(applications::Column::Status.eq(status))
        .order_by_desc(applications::Column::SubmittedAt)
        .all(db)
        .await?)
}

async fn own_office_reviews(
    db: &DatabaseConnection,
    reviewer_id: Uuid,
    decision: ReviewDecision,
) -> Result<Vec<reviews::Model>, AppError> {
    Ok(reviews::Entity::find()
        .filter(reviews::Column::ReviewerId.eq(reviewer_id))
        .filter(reviews::Column::Stage.eq(ReviewStage::ResearchOffice))
        .filter(reviews::Column::Decision.eq(decision))
        .order_by_desc(reviews::Column::ReviewedAt)
        .all(db)
        .await?)
}

async fn pack(
    db: &DatabaseConnection,
    apps: &[applications::Model],
    own_reviews: Option<&[reviews::Model]>,
) -> Result<Vec<OfficeListItem>, AppError> {
    let names = department_names(db).await?;
    let totals = budget_totals(db, apps).await?;
    let ids: Vec<Uuid> = apps.iter().map(|app| app.id).collect();
    let progress = progress_for(db, &ids).await?;
    if let Some(rows) = own_reviews {
        let by_id: HashMap<Uuid, &applications::Model> =
            apps.iter().map(|app| (app.id, app)).collect();
        return Ok(rows
            .iter()
            .filter_map(|review| {
                by_id
                    .get(&review.application_id)
                    .map(|app| to_item(app, &names, &totals, &progress, Some(review)))
            })
            .collect());
    }
    Ok(apps
        .iter()
        .map(|app| to_item(app, &names, &totals, &progress, None))
        .collect())
}

#[derive(Default)]
struct Progress {
    assigned: usize,
    reviewed: usize,
    average: Option<String>,
}

async fn progress_for(
    db: &DatabaseConnection,
    ids: &[Uuid],
) -> Result<HashMap<Uuid, Progress>, AppError> {
    let mut progress = HashMap::new();
    if ids.is_empty() {
        return Ok(progress);
    }
    let assignments = expert_assignments::Entity::find()
        .filter(expert_assignments::Column::ApplicationId.is_in(ids.to_vec()))
        .all(db)
        .await?;
    let expert_reviews = reviews::Entity::find()
        .filter(reviews::Column::ApplicationId.is_in(ids.to_vec()))
        .filter(reviews::Column::Stage.eq(ReviewStage::Expert))
        .all(db)
        .await?;
    for assignment in &assignments {
        progress
            .entry(assignment.application_id)
            .or_default()
            .assigned += 1;
    }
    let mut scores: HashMap<Uuid, Vec<sea_orm::prelude::Decimal>> = HashMap::new();
    for review in &expert_reviews {
        let entry = progress.entry(review.application_id).or_default();
        entry.reviewed += 1;
        if let Some(score) = review.score {
            scores.entry(review.application_id).or_default().push(score);
        }
    }
    for (id, values) in scores {
        if let Some(entry) = progress.get_mut(&id) {
            entry.average = average_decimals(&values);
        }
    }
    Ok(progress)
}

fn average_decimals(values: &[sea_orm::prelude::Decimal]) -> Option<String> {
    if values.is_empty() {
        return None;
    }
    let count = sea_orm::prelude::Decimal::from(values.len() as i64);
    let sum = values.iter().copied().sum::<sea_orm::prelude::Decimal>();
    Some(forms::money_text(sum / count))
}

fn average_of<'a>(values: impl Iterator<Item = &'a str>) -> Option<String> {
    let parsed: Vec<sea_orm::prelude::Decimal> =
        values.filter_map(|value| value.parse().ok()).collect();
    average_decimals(&parsed)
}

fn to_item(
    app: &applications::Model,
    names: &HashMap<Uuid, String>,
    totals: &HashMap<Uuid, String>,
    progress: &HashMap<Uuid, Progress>,
    review: Option<&reviews::Model>,
) -> OfficeListItem {
    let stats = progress.get(&app.id);
    OfficeListItem {
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
        code: app.code.clone(),
        assigned_count: stats.map(|item| item.assigned).unwrap_or(0),
        reviewed_count: stats.map(|item| item.reviewed).unwrap_or(0),
        average_score: stats.and_then(|item| item.average.clone()),
        decision: review.map(|row| decision_code(&row.decision).to_owned()),
        opinion: review.and_then(|row| row.opinion.clone()),
        reviewed_at: review.and_then(|row| row.reviewed_at.map(|value| value.to_rfc3339())),
        conclusion_opinion: app.conclusion_opinion.clone(),
    }
}

async fn assignment_views(
    db: &DatabaseConnection,
    id: Uuid,
) -> Result<Vec<AssignmentView>, AppError> {
    let assignments = expert_assignments::Entity::find()
        .filter(expert_assignments::Column::ApplicationId.eq(id))
        .order_by_asc(expert_assignments::Column::AssignedAt)
        .all(db)
        .await?;
    let expert_reviews = reviews::Entity::find()
        .filter(reviews::Column::ApplicationId.eq(id))
        .filter(reviews::Column::Stage.eq(ReviewStage::Expert))
        .all(db)
        .await?;
    let mut views = Vec::new();
    for assignment in assignments {
        let expert = users::Entity::find_by_id(assignment.expert_id)
            .one(db)
            .await?;
        let review = expert_reviews
            .iter()
            .find(|row| row.assignment_id == Some(assignment.id));
        views.push(AssignmentView {
            id: assignment.id,
            expert_id: assignment.expert_id,
            expert_name: expert
                .as_ref()
                .map(|row| row.real_name.clone())
                .unwrap_or_else(|| "未知专家".to_owned()),
            research_expertise: expert.and_then(|row| row.research_expertise),
            reviewed: review.is_some(),
            decision: review.map(|row| decision_code(&row.decision).to_owned()),
            opinion: review.and_then(|row| row.opinion.clone()),
            score: review.and_then(|row| row.score.map(forms::money_text)),
            reviewed_at: review.and_then(|row| row.reviewed_at.map(|value| value.to_rfc3339())),
        });
    }
    Ok(views)
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

async fn load_in<C>(connection: &C, id: Uuid) -> Result<applications::Model, AppError>
where
    C: ConnectionTrait,
{
    applications::Entity::find_by_id(id)
        .one(connection)
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
    use super::{
        ConclusionInput, OfficeDecisionInput, check_conclusion, check_office_decision,
        person_conflict,
    };
    use crate::error::AppError;

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
    fn approve_requires_a_cover_code() {
        let error = check_office_decision(&OfficeDecisionInput {
            decision: "approved".to_owned(),
            opinion: None,
            code: Some("  ".to_owned()),
        })
        .expect_err("空编号");
        assert_eq!(message(error), "通过时必须填写封面编号");
    }

    #[test]
    fn return_requires_opinion() {
        let error = check_office_decision(&OfficeDecisionInput {
            decision: "returned".to_owned(),
            opinion: None,
            code: None,
        })
        .expect_err("空意见");
        assert_eq!(message(error), "退回或不通过必须填写意见");
    }

    #[test]
    fn conclusion_requires_an_explanation() {
        let error = check_conclusion(&ConclusionInput {
            decision: "established".to_owned(),
            opinion: Some(" ".to_owned()),
        })
        .expect_err("空说明");
        assert_eq!(message(error), "请填写结论说明");
    }

    #[test]
    fn expert_cannot_be_the_principal_member_or_recommender() {
        assert!(person_conflict("赵明远", "赵明远", &[], None));
        assert!(person_conflict("陈霞", "赵明远", &["陈霞"], None));
        assert!(person_conflict("王海宁", "赵明远", &[], Some("王海宁")));
        assert!(!person_conflict(
            "匿名专家E-016",
            "赵明远",
            &["陈霞"],
            Some("王海宁")
        ));
    }
}

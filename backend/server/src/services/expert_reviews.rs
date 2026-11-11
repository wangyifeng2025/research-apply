//! 专家盲审。列表和详情都不包含负责人、部门、参加者和推荐人。

use std::str::FromStr;

use entity::application_budgets;
use entity::applications;
use entity::budget_subjects;
use entity::expert_assignments;
use entity::ns_equipments;
use entity::ns_indicators;
use entity::ns_profiles;
use entity::ns_research_contents;
use entity::ns_schedules;
use entity::reviews;
use entity::sea_orm_active_enums::{OutputKind, ReviewDecision, ReviewStage, UserRole};
use entity::ss_designs;
use entity::ss_expected_outputs;
use entity::ss_foundations;
use entity::ss_prior_achievements;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, ConnectionTrait, DatabaseConnection, EntityTrait, QueryFilter,
    QueryOrder, Set, TransactionTrait,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use uuid::Uuid;

use crate::auth::{CurrentUser, require_role};
use crate::error::{AppError, FieldError};
use crate::services::applications::{self as forms};
use crate::services::applications::{
    BudgetView, DesignView, IndicatorView, ScheduleView, empty_indicators, indicator_view,
};
use crate::support::audit::{AuditEntry, record};
use crate::support::time::now;

#[derive(Debug, Deserialize)]
pub struct ExpertReviewInput {
    pub decision: String,
    pub opinion: Option<String>,
    pub score: String,
}

#[derive(Debug, Serialize)]
pub struct ExpertListItem {
    pub id: Uuid,
    pub project_class: String,
    pub title: Option<String>,
    pub decision: Option<String>,
    pub score: Option<String>,
    pub opinion: Option<String>,
    pub reviewed_at: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ExpertDetail {
    pub application: BlindApplication,
    pub review: Option<SubmittedReview>,
}

#[derive(Debug, Serialize)]
pub struct SubmittedReview {
    pub decision: String,
    pub score: String,
    pub opinion: Option<String>,
    pub reviewed_at: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct BlindApplication {
    pub id: Uuid,
    pub project_class: String,
    pub title: Option<String>,
    pub keywords: Option<String>,
    pub budget: BudgetView,
    pub ss_achievements: Vec<BlindAchievement>,
    pub ss_design: DesignView,
    pub ss_foundation: BlindFoundation,
    pub ss_stage_outputs: Vec<BlindOutput>,
    pub ss_final_outputs: Vec<BlindOutput>,
    pub ns_profile: Option<String>,
    pub ns_content: Option<String>,
    pub ns_indicators: IndicatorView,
    pub ns_schedules: Vec<ScheduleView>,
    pub ns_equipments: Vec<BlindEquipment>,
    pub attachments: Vec<crate::services::attachments::AttachmentView>,
}

#[derive(Debug, Serialize)]
pub struct BlindAchievement {
    pub title: String,
    pub outcome_form: Option<String>,
    pub publication: Option<String>,
    pub published_on: Option<String>,
}

#[derive(Debug, Serialize, Default)]
pub struct BlindFoundation {
    pub research_basis: Option<String>,
    pub undertaken_projects: Option<String>,
    pub support_conditions: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct BlindOutput {
    pub complete_on: Option<String>,
    pub title: String,
    pub outcome_form: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct BlindEquipment {
    pub name_spec: String,
    pub quantity: i32,
    pub unit_price: String,
    pub amount: String,
    pub funding_source: Option<String>,
    pub purpose: Option<String>,
}

pub async fn queue(
    db: &DatabaseConnection,
    user: &CurrentUser,
    scope: &str,
) -> Result<Vec<ExpertListItem>, AppError> {
    require_role(user, UserRole::Expert)?;
    if scope != "pending" && scope != "reviewed" {
        return Err(AppError::BadRequest("审核列表范围无效".to_owned()));
    }
    let assignments = expert_assignments::Entity::find()
        .filter(expert_assignments::Column::ExpertId.eq(user.id))
        .order_by_desc(expert_assignments::Column::AssignedAt)
        .all(db)
        .await?;
    if assignments.is_empty() {
        return Ok(Vec::new());
    }
    let ids: Vec<Uuid> = assignments.iter().map(|row| row.application_id).collect();
    let apps = applications::Entity::find()
        .filter(applications::Column::Id.is_in(ids))
        .all(db)
        .await?;
    let submitted = reviews::Entity::find()
        .filter(reviews::Column::ReviewerId.eq(user.id))
        .filter(reviews::Column::Stage.eq(ReviewStage::Expert))
        .all(db)
        .await?;
    let mut items = Vec::new();
    for assignment in assignments {
        let Some(app) = apps.iter().find(|row| row.id == assignment.application_id) else {
            continue;
        };
        let review = submitted
            .iter()
            .find(|row| row.assignment_id == Some(assignment.id));
        let pending = review.is_none();
        if (scope == "pending" && !pending) || (scope == "reviewed" && pending) {
            continue;
        }
        items.push(ExpertListItem {
            id: app.id,
            project_class: forms::class_code(&app.project_class).to_owned(),
            title: app.title.clone(),
            decision: review.map(|row| decision_code(&row.decision).to_owned()),
            score: review.and_then(|row| row.score.map(forms::money_text)),
            opinion: review.and_then(|row| row.opinion.clone()),
            reviewed_at: review.and_then(|row| row.reviewed_at.map(|value| value.to_rfc3339())),
        });
    }
    Ok(items)
}

pub async fn open(
    db: &DatabaseConnection,
    user: &CurrentUser,
    id: Uuid,
) -> Result<ExpertDetail, AppError> {
    require_role(user, UserRole::Expert)?;
    let assignment = assigned_to(db, user.id, id).await?;
    let review = reviews::Entity::find()
        .filter(reviews::Column::AssignmentId.eq(assignment.id))
        .one(db)
        .await?;
    Ok(ExpertDetail {
        application: load_blind(db, id).await?,
        review: review.map(|row| SubmittedReview {
            decision: decision_code(&row.decision).to_owned(),
            score: row
                .score
                .map(forms::money_text)
                .unwrap_or_else(|| "0.00".to_owned()),
            opinion: row.opinion,
            reviewed_at: row.reviewed_at.map(|value| value.to_rfc3339()),
        }),
    })
}

pub async fn submit(
    db: &DatabaseConnection,
    user: &CurrentUser,
    id: Uuid,
    input: ExpertReviewInput,
) -> Result<SubmittedReview, AppError> {
    require_role(user, UserRole::Expert)?;
    let (decision, opinion, score) = check_expert_review(&input)?;
    let txn = db.begin().await?;
    let saved = match write_review(&txn, user, id, decision, opinion, score).await {
        Ok(review) => review,
        Err(error) => {
            if let Err(rollback_error) = txn.rollback().await {
                tracing::error!(error = %rollback_error, "专家评审事务回滚失败");
            }
            return Err(error);
        }
    };
    txn.commit().await?;
    Ok(SubmittedReview {
        decision: decision_code(&saved.decision).to_owned(),
        score: saved
            .score
            .map(forms::money_text)
            .unwrap_or_else(|| "0.00".to_owned()),
        opinion: saved.opinion,
        reviewed_at: saved.reviewed_at.map(|value| value.to_rfc3339()),
    })
}

async fn write_review<C>(
    txn: &C,
    user: &CurrentUser,
    id: Uuid,
    decision: ReviewDecision,
    opinion: Option<String>,
    score: sea_orm::prelude::Decimal,
) -> Result<reviews::Model, AppError>
where
    C: ConnectionTrait,
{
    let assignment = assigned_to(txn, user.id, id).await?;
    let existing = reviews::Entity::find()
        .filter(reviews::Column::AssignmentId.eq(assignment.id))
        .one(txn)
        .await?;
    if existing.is_some() {
        return Err(AppError::Conflict("评审已经提交，不能修改".to_owned()));
    }
    let timestamp = now();
    let saved = reviews::ActiveModel {
        id: Set(Uuid::new_v4()),
        application_id: Set(id),
        stage: Set(ReviewStage::Expert),
        reviewer_id: Set(user.id),
        assignment_id: Set(Some(assignment.id)),
        decision: Set(decision.clone()),
        opinion: Set(opinion.clone()),
        score: Set(Some(score)),
        facts_accurate: Set(None),
        team_qualified: Set(None),
        resources_ready: Set(None),
        agree_to_manage: Set(None),
        is_blind: Set(true),
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
            action: "expert_review",
            target_type: "application",
            target_id: Some(id),
            detail: Some(json!({
                "assignment_id": assignment.id,
                "decision": decision_code(&decision),
                "score": forms::money_text(score),
            })),
            ip_address: None,
            user_agent: None,
        },
    )
    .await?;
    Ok(saved)
}

async fn assigned_to<C>(
    connection: &C,
    expert_id: Uuid,
    application_id: Uuid,
) -> Result<expert_assignments::Model, AppError>
where
    C: ConnectionTrait,
{
    expert_assignments::Entity::find()
        .filter(expert_assignments::Column::ExpertId.eq(expert_id))
        .filter(expert_assignments::Column::ApplicationId.eq(application_id))
        .one(connection)
        .await?
        .ok_or_else(|| AppError::NotFound("申报书不存在".to_owned()))
}

async fn load_blind(db: &DatabaseConnection, id: Uuid) -> Result<BlindApplication, AppError> {
    let app = applications::Entity::find_by_id(id)
        .one(db)
        .await?
        .ok_or_else(|| AppError::NotFound("申报书不存在".to_owned()))?;
    let family = forms::family_of(&app.project_class);
    let subjects = budget_subjects::Entity::find()
        .filter(budget_subjects::Column::AppliesTo.eq(family))
        .order_by_asc(budget_subjects::Column::SortOrder)
        .all(db)
        .await?;
    let budgets = application_budgets::Entity::find()
        .filter(application_budgets::Column::ApplicationId.eq(id))
        .all(db)
        .await?;
    let direct = forms::budget_total(
        &subjects
            .iter()
            .filter(|subject| subject.group_name == "direct")
            .cloned()
            .collect::<Vec<_>>(),
        &budgets,
    );
    let total = forms::budget_total(&subjects, &budgets);
    let achievements = ss_prior_achievements::Entity::find()
        .filter(ss_prior_achievements::Column::ApplicationId.eq(id))
        .order_by_asc(ss_prior_achievements::Column::SortOrder)
        .all(db)
        .await?;
    let design = ss_designs::Entity::find_by_id(id).one(db).await?;
    let foundation = ss_foundations::Entity::find_by_id(id).one(db).await?;
    let outputs = ss_expected_outputs::Entity::find()
        .filter(ss_expected_outputs::Column::ApplicationId.eq(id))
        .order_by_asc(ss_expected_outputs::Column::SortOrder)
        .all(db)
        .await?;
    let profile = ns_profiles::Entity::find_by_id(id).one(db).await?;
    let content = ns_research_contents::Entity::find_by_id(id).one(db).await?;
    let indicators = ns_indicators::Entity::find_by_id(id).one(db).await?;
    let schedules = ns_schedules::Entity::find()
        .filter(ns_schedules::Column::ApplicationId.eq(id))
        .order_by_asc(ns_schedules::Column::SortOrder)
        .all(db)
        .await?;
    let equipments = ns_equipments::Entity::find()
        .filter(ns_equipments::Column::ApplicationId.eq(id))
        .order_by_asc(ns_equipments::Column::SortOrder)
        .all(db)
        .await?;
    Ok(BlindApplication {
        id: app.id,
        project_class: forms::class_code(&app.project_class).to_owned(),
        title: app.title,
        keywords: app.keywords,
        budget: BudgetView {
            lines: subjects
                .iter()
                .map(|subject| forms::BudgetLineView {
                    amount: budgets
                        .iter()
                        .find(|line| line.subject_code == subject.code)
                        .map(|line| forms::money_text(line.amount))
                        .unwrap_or_else(|| "0.00".to_owned()),
                    code: subject.code.clone(),
                    name: subject.name.clone(),
                    group_name: subject.group_name.clone(),
                    parent_code: subject.parent_code.clone(),
                    fillable: subject.fillable,
                })
                .collect(),
            direct_total: forms::money_text(direct),
            indirect_total: forms::money_text(total - direct),
            total: forms::money_text(total),
        },
        ss_achievements: achievements
            .into_iter()
            .map(|row| BlindAchievement {
                title: row.title,
                outcome_form: row.outcome_form,
                publication: row.publication,
                published_on: row.published_on,
            })
            .collect(),
        ss_design: design
            .map(|row| DesignView {
                topic_basis: row.topic_basis,
                research_content: row.research_content,
                methodology: row.methodology,
                innovation: row.innovation,
                expected_outcome: row.expected_outcome,
                reference_list: row.reference_list,
            })
            .unwrap_or_default(),
        ss_foundation: foundation
            .map(|row| BlindFoundation {
                research_basis: row.research_basis,
                undertaken_projects: row.undertaken_projects,
                support_conditions: row.support_conditions,
            })
            .unwrap_or_default(),
        ss_stage_outputs: outputs
            .iter()
            .filter(|row| row.kind == OutputKind::Stage)
            .map(blind_output)
            .collect(),
        ss_final_outputs: outputs
            .iter()
            .filter(|row| row.kind == OutputKind::Final)
            .map(blind_output)
            .collect(),
        ns_profile: profile.and_then(|row| row.summary),
        ns_content: content.and_then(|row| row.content),
        ns_indicators: indicators
            .map(indicator_view)
            .unwrap_or_else(empty_indicators),
        ns_schedules: schedules
            .into_iter()
            .map(|row| ScheduleView {
                start_date: row
                    .start_date
                    .map(|date| date.format("%Y-%m-%d").to_string()),
                end_date: row.end_date.map(|date| date.format("%Y-%m-%d").to_string()),
                goal: row.goal,
            })
            .collect(),
        ns_equipments: equipments
            .into_iter()
            .map(|row| BlindEquipment {
                name_spec: row.name_spec,
                quantity: row.quantity.unwrap_or(0),
                unit_price: row
                    .unit_price
                    .map(forms::money_text)
                    .unwrap_or_else(|| "0.00".to_owned()),
                amount: row
                    .amount
                    .map(forms::money_text)
                    .unwrap_or_else(|| "0.00".to_owned()),
                funding_source: row.funding_source,
                purpose: row.purpose,
            })
            .collect(),
        attachments: crate::services::attachments::list_views(db, id, false).await?,
    })
}

fn blind_output(row: &ss_expected_outputs::Model) -> BlindOutput {
    BlindOutput {
        complete_on: row.complete_on.clone(),
        title: row.title.clone(),
        outcome_form: row.outcome_form.clone(),
    }
}

fn check_expert_review(
    input: &ExpertReviewInput,
) -> Result<(ReviewDecision, Option<String>, sea_orm::prelude::Decimal), AppError> {
    let decision = match input.decision.as_str() {
        "approved" => ReviewDecision::Approved,
        "rejected" => ReviewDecision::Rejected,
        "returned" => return Err(invalid("decision", "专家评审不能退回")),
        _ => return Err(invalid("decision", "评审结论只能是建议立项或不建议立项")),
    };
    let opinion = input
        .opinion
        .as_deref()
        .map(str::trim)
        .filter(|text| !text.is_empty())
        .map(str::to_owned);
    if opinion.is_none() {
        return Err(invalid("opinion", "请填写评审意见"));
    }
    Ok((decision, opinion, parse_score(&input.score)?))
}

fn parse_score(raw: &str) -> Result<sea_orm::prelude::Decimal, AppError> {
    let text = raw.trim();
    let score = sea_orm::prelude::Decimal::from_str(text)
        .map_err(|_| invalid("score", "评分必须是 0 到 100 的数字"))?;
    if score.scale() > 2 {
        return Err(invalid("score", "评分最多保留两位小数"));
    }
    if score < sea_orm::prelude::Decimal::ZERO || score > sea_orm::prelude::Decimal::from(100) {
        return Err(invalid("score", "评分必须在 0 到 100 之间"));
    }
    Ok(score)
}

fn decision_code(decision: &ReviewDecision) -> &'static str {
    match decision {
        ReviewDecision::Pending => "pending",
        ReviewDecision::Approved => "approved",
        ReviewDecision::Rejected => "rejected",
        ReviewDecision::Returned => "returned",
    }
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
        BlindAchievement, BlindApplication, BlindEquipment, BlindFoundation, BlindOutput,
        ExpertReviewInput, check_expert_review,
    };
    use crate::error::AppError;
    use crate::services::applications::{BudgetView, DesignView, empty_indicators};
    use serde_json::Value;
    use std::collections::HashSet;
    use uuid::Uuid;

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

    fn keys(value: &Value, found: &mut HashSet<String>) {
        match value {
            Value::Object(map) => {
                for (key, child) in map {
                    found.insert(key.clone());
                    keys(child, found);
                }
            }
            Value::Array(items) => {
                for item in items {
                    keys(item, found);
                }
            }
            _ => {}
        }
    }

    #[test]
    fn serialized_blind_application_omits_identity() {
        let view = BlindApplication {
            id: Uuid::nil(),
            project_class: "social".to_owned(),
            title: Some("匿名题目".to_owned()),
            keywords: None,
            budget: BudgetView {
                lines: Vec::new(),
                direct_total: "0.00".to_owned(),
                indirect_total: "0.00".to_owned(),
                total: "0.00".to_owned(),
            },
            ss_achievements: vec![BlindAchievement {
                title: "成果".to_owned(),
                outcome_form: None,
                publication: None,
                published_on: None,
            }],
            ss_design: DesignView::default(),
            ss_foundation: BlindFoundation::default(),
            ss_stage_outputs: vec![BlindOutput {
                complete_on: None,
                title: "阶段".to_owned(),
                outcome_form: None,
            }],
            ss_final_outputs: Vec::new(),
            ns_profile: None,
            ns_content: None,
            ns_indicators: empty_indicators(),
            ns_schedules: Vec::new(),
            ns_equipments: vec![BlindEquipment {
                name_spec: "设备".to_owned(),
                quantity: 1,
                unit_price: "5000.00".to_owned(),
                amount: "5000.00".to_owned(),
                funding_source: None,
                purpose: None,
            }],
            attachments: Vec::new(),
        };
        let value = serde_json::to_value(view).expect("匿名视图可以序列化");
        let mut found = HashSet::new();
        keys(&value, &mut found);
        for forbidden in [
            "principal_name",
            "department_name",
            "department_id",
            "members",
            "recommender",
            "mobile_phone",
            "academic_resume",
            "owner_name",
            "researchers",
            "reviewer_id",
            "real_name",
        ] {
            assert!(!found.contains(forbidden), "{forbidden} 不应出现在匿名视图");
        }
    }

    #[test]
    fn expert_score_stays_within_one_hundred_and_two_decimals() {
        let accepted = check_expert_review(&ExpertReviewInput {
            decision: "approved".to_owned(),
            opinion: Some("建议立项".to_owned()),
            score: "86.5".to_owned(),
        });
        assert!(accepted.is_ok());
        let overflow = check_expert_review(&ExpertReviewInput {
            decision: "rejected".to_owned(),
            opinion: Some("不建议".to_owned()),
            score: "100.01".to_owned(),
        })
        .expect_err("超分");
        assert_eq!(message(overflow), "评分必须在 0 到 100 之间");
        let precise = check_expert_review(&ExpertReviewInput {
            decision: "approved".to_owned(),
            opinion: Some("建议立项".to_owned()),
            score: "86.555".to_owned(),
        })
        .expect_err("小数位");
        assert_eq!(message(precise), "评分最多保留两位小数");
    }

    #[test]
    fn expert_cannot_return_an_application() {
        let error = check_expert_review(&ExpertReviewInput {
            decision: "returned".to_owned(),
            opinion: Some("退回".to_owned()),
            score: "80".to_owned(),
        })
        .expect_err("退回");
        assert_eq!(message(error), "专家评审不能退回");
    }
}

//! 审核时间线：按时间返回系审核、科研处审核、专家评审和立项结论。
//!
//! 申请人与系审核人看到的专家写成「专家 1 / 2 / 3」，响应里没有 `reviewer_id`。
//! 科研处和管理员可以看到专家姓名。专家本人不能通过这个接口读取审核链。

use std::collections::HashMap;

use chrono::{DateTime, FixedOffset};
use entity::applications;
use entity::reviews;
use entity::sea_orm_active_enums::{ApplicationStatus, ReviewDecision, ReviewStage, UserRole};
use entity::users;
use sea_orm::prelude::Decimal;
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use serde::Serialize;
use uuid::Uuid;

use crate::auth::CurrentUser;
use crate::error::AppError;
use crate::services::applications::{money_text, status_code};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ExpertNames {
    Hidden,
    Visible,
}

struct TimelineReview {
    stage: ReviewStage,
    reviewer_name: String,
    decision: ReviewDecision,
    opinion: Option<String>,
    score: Option<Decimal>,
    reviewed_at: Option<DateTime<FixedOffset>>,
    facts_accurate: Option<bool>,
    team_qualified: Option<bool>,
    resources_ready: Option<bool>,
    agree_to_manage: Option<bool>,
}

struct TimelineConclusion {
    actor: String,
    decision: String,
    opinion: Option<String>,
    at: DateTime<FixedOffset>,
}

#[derive(Debug, Serialize, PartialEq, Eq)]
pub struct TimelineEvent {
    pub stage: String,
    pub actor: String,
    pub decision: String,
    pub opinion: Option<String>,
    pub score: Option<String>,
    pub reviewed_at: String,
    pub facts_accurate: Option<bool>,
    pub team_qualified: Option<bool>,
    pub resources_ready: Option<bool>,
    pub agree_to_manage: Option<bool>,
}

pub async fn list(
    db: &DatabaseConnection,
    user: &CurrentUser,
    id: Uuid,
) -> Result<Vec<TimelineEvent>, AppError> {
    let app = applications::Entity::find_by_id(id)
        .one(db)
        .await?
        .ok_or_else(|| AppError::NotFound("申报书不存在".to_owned()))?;
    let experts = expert_names_for(user, &app)?;
    let rows = reviews::Entity::find()
        .filter(reviews::Column::ApplicationId.eq(id))
        .all(db)
        .await?;
    let mut user_ids: Vec<Uuid> = rows.iter().map(|row| row.reviewer_id).collect();
    if let Some(concluded_by) = app.concluded_by {
        user_ids.push(concluded_by);
    }
    user_ids.sort_unstable();
    user_ids.dedup();
    let names = reviewer_names(db, &user_ids).await?;
    let records = rows
        .into_iter()
        .map(|row| TimelineReview {
            stage: row.stage,
            reviewer_name: names
                .get(&row.reviewer_id)
                .cloned()
                .unwrap_or_else(|| "审核人".to_owned()),
            decision: row.decision,
            opinion: row.opinion,
            score: row.score,
            reviewed_at: row.reviewed_at,
            facts_accurate: row.facts_accurate,
            team_qualified: row.team_qualified,
            resources_ready: row.resources_ready,
            agree_to_manage: row.agree_to_manage,
        })
        .collect();
    Ok(assemble(records, conclusion_of(&app, &names), experts))
}

fn expert_names_for(
    user: &CurrentUser,
    app: &applications::Model,
) -> Result<ExpertNames, AppError> {
    if user.has_role(UserRole::Admin) || user.has_role(UserRole::ResearchOffice) {
        return Ok(ExpertNames::Visible);
    }
    if app.principal_id == user.id {
        return Ok(ExpertNames::Hidden);
    }
    if user.has_role(UserRole::DepartmentReviewer)
        && user.department_id.is_some()
        && user.department_id == app.department_id
    {
        return Ok(ExpertNames::Hidden);
    }
    if user.has_role(UserRole::DepartmentReviewer) {
        return Err(AppError::Forbidden(
            "只能查看本部门申报书的审核进度".to_owned(),
        ));
    }
    Err(AppError::Forbidden(
        "无权查看这份申报书的审核进度".to_owned(),
    ))
}

async fn reviewer_names(
    db: &DatabaseConnection,
    ids: &[Uuid],
) -> Result<HashMap<Uuid, String>, AppError> {
    if ids.is_empty() {
        return Ok(HashMap::new());
    }
    let rows = users::Entity::find()
        .filter(users::Column::Id.is_in(ids.to_vec()))
        .all(db)
        .await?;
    Ok(rows
        .into_iter()
        .map(|row| (row.id, row.real_name))
        .collect())
}

fn conclusion_of(
    app: &applications::Model,
    names: &HashMap<Uuid, String>,
) -> Option<TimelineConclusion> {
    let at = app.concluded_at?;
    if !matches!(
        app.status,
        ApplicationStatus::Established | ApplicationStatus::ExpertRejected
    ) {
        return None;
    }
    let actor = app
        .concluded_by
        .and_then(|id| names.get(&id).cloned())
        .unwrap_or_else(|| "科研处".to_owned());
    Some(TimelineConclusion {
        actor,
        decision: status_code(&app.status).to_owned(),
        opinion: app.conclusion_opinion.clone(),
        at,
    })
}

fn assemble(
    reviews: Vec<TimelineReview>,
    conclusion: Option<TimelineConclusion>,
    experts: ExpertNames,
) -> Vec<TimelineEvent> {
    let mut timed: Vec<(DateTime<FixedOffset>, TimelineEvent)> = reviews
        .into_iter()
        .filter(|row| row.decision != ReviewDecision::Pending)
        .filter_map(|row| {
            let at = row.reviewed_at?;
            Some((at, review_event(row, at)))
        })
        .collect();
    if let Some(item) = conclusion {
        timed.push((item.at, conclusion_event(item)));
    }
    timed.sort_by_key(|(at, _)| *at);
    if experts == ExpertNames::Hidden {
        let mut expert_no = 0_u32;
        for (_, event) in &mut timed {
            if event.stage == "expert" {
                expert_no += 1;
                event.actor = format!("专家 {expert_no}");
            }
        }
    }
    timed.into_iter().map(|(_, event)| event).collect()
}

fn review_event(row: TimelineReview, at: DateTime<FixedOffset>) -> TimelineEvent {
    let department = row.stage == ReviewStage::Department;
    let expert = row.stage == ReviewStage::Expert;
    TimelineEvent {
        stage: stage_code(&row.stage).to_owned(),
        actor: row.reviewer_name,
        decision: decision_code(&row.decision).to_owned(),
        opinion: row.opinion,
        score: expert.then(|| row.score.map(money_text)).flatten(),
        reviewed_at: at.to_rfc3339(),
        facts_accurate: department.then_some(row.facts_accurate).flatten(),
        team_qualified: department.then_some(row.team_qualified).flatten(),
        resources_ready: department.then_some(row.resources_ready).flatten(),
        agree_to_manage: department.then_some(row.agree_to_manage).flatten(),
    }
}

fn conclusion_event(item: TimelineConclusion) -> TimelineEvent {
    TimelineEvent {
        stage: "conclusion".to_owned(),
        actor: item.actor,
        decision: item.decision,
        opinion: item.opinion,
        score: None,
        reviewed_at: item.at.to_rfc3339(),
        facts_accurate: None,
        team_qualified: None,
        resources_ready: None,
        agree_to_manage: None,
    }
}

fn stage_code(stage: &ReviewStage) -> &'static str {
    match stage {
        ReviewStage::Department => "department",
        ReviewStage::ResearchOffice => "research_office",
        ReviewStage::Expert => "expert",
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

#[cfg(test)]
mod tests {
    use super::*;

    fn at(raw: &str) -> DateTime<FixedOffset> {
        DateTime::parse_from_rfc3339(raw).expect("时间")
    }

    fn review(
        stage: ReviewStage,
        name: &str,
        decision: ReviewDecision,
        opinion: &str,
        score: Option<&str>,
        when: &str,
    ) -> TimelineReview {
        TimelineReview {
            stage,
            reviewer_name: name.to_owned(),
            decision,
            opinion: Some(opinion.to_owned()),
            score: score.map(|value| value.parse().expect("分数")),
            reviewed_at: Some(at(when)),
            facts_accurate: Some(true),
            team_qualified: Some(true),
            resources_ready: Some(false),
            agree_to_manage: Some(true),
        }
    }

    fn sample() -> Vec<TimelineReview> {
        vec![
            review(
                ReviewStage::Department,
                "李文博",
                ReviewDecision::Approved,
                "同意上报",
                None,
                "2026-10-01T09:00:00+08:00",
            ),
            review(
                ReviewStage::ResearchOffice,
                "王海宁",
                ReviewDecision::Returned,
                "请补充预算",
                None,
                "2026-10-02T09:00:00+08:00",
            ),
            review(
                ReviewStage::Expert,
                "陈一",
                ReviewDecision::Approved,
                "建议立项",
                Some("70.50"),
                "2026-10-05T09:00:00+08:00",
            ),
            review(
                ReviewStage::Expert,
                "林二",
                ReviewDecision::Rejected,
                "论证不足",
                Some("90"),
                "2026-10-04T09:00:00+08:00",
            ),
            TimelineReview {
                stage: ReviewStage::Expert,
                reviewer_name: "周三角".to_owned(),
                decision: ReviewDecision::Pending,
                opinion: None,
                score: Some("10".parse().expect("分数")),
                reviewed_at: Some(at("2026-10-06T09:00:00+08:00")),
                facts_accurate: None,
                team_qualified: None,
                resources_ready: None,
                agree_to_manage: None,
            },
        ]
    }

    fn conclusion() -> TimelineConclusion {
        TimelineConclusion {
            actor: "王海宁".to_owned(),
            decision: "established".to_owned(),
            opinion: Some("同意立项".to_owned()),
            at: at("2026-10-07T09:00:00+08:00"),
        }
    }

    #[test]
    fn applicant_timeline_numbers_experts_and_omits_identity() {
        let events = assemble(sample(), Some(conclusion()), ExpertNames::Hidden);
        let actors: Vec<&str> = events.iter().map(|event| event.actor.as_str()).collect();
        assert_eq!(actors, ["李文博", "王海宁", "专家 1", "专家 2", "王海宁"]);
        assert_eq!(events[2].score.as_deref(), Some("90.00"));
        assert_eq!(events[2].opinion.as_deref(), Some("论证不足"));
        assert_eq!(events[3].score.as_deref(), Some("70.50"));
        assert_eq!(events[4].stage, "conclusion");
        assert_eq!(events[4].decision, "established");
        assert_eq!(events[0].facts_accurate, Some(true));
        assert_eq!(events[0].resources_ready, Some(false));
        assert_eq!(events[2].facts_accurate, None);
        let serialized = serde_json::to_string(&events).expect("时间线可以序列化");
        for leaked in ["reviewer_id", "陈一", "林二", "周三角", "real_name"] {
            assert!(
                !serialized.contains(leaked),
                "{leaked} 不应出现在申请人时间线"
            );
        }
    }

    #[test]
    fn office_timeline_keeps_expert_names() {
        let events = assemble(sample(), None, ExpertNames::Visible);
        assert_eq!(events[2].actor, "林二");
        assert_eq!(events[3].actor, "陈一");
        assert!(events.iter().all(|event| event.stage != "conclusion"));
    }
}

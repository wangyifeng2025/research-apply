//! 教师申报书：新建、按栏目保存、提交、撤回、删除草稿。
//!
//! 保存策略：栏目在「保存草稿」或「下一步」时整组写入。参加者、成果、进度、设备
//! 不在新增行时落库，而是随所在栏目在同一事务里先删后插。删除草稿只删主表，
//! 子表靠外键 `ON DELETE CASCADE` 一起删除。

use chrono::NaiveDate;
use entity::application_budgets;
use entity::application_cycles;
use entity::application_members;
use entity::applications;
use entity::budget_subjects;
use entity::departments;
use entity::ns_equipments;
use entity::ns_indicators;
use entity::ns_profiles;
use entity::ns_research_contents;
use entity::ns_schedules;
use entity::recommenders;
use entity::reviews;
use entity::sea_orm_active_enums::{
    ApplicationStatus, BudgetFamily, CycleStatus, OutputKind, ProjectClass, ReviewStage, UserRole,
};
use entity::ss_designs;
use entity::ss_expected_outputs;
use entity::ss_foundations;
use entity::ss_prior_achievements;
use entity::users;
use sea_orm::sea_query::Expr;
use sea_orm::{
    ActiveEnum, ActiveModelTrait, ColumnTrait, DatabaseConnection, DatabaseTransaction, EntityTrait,
    QueryFilter, QueryOrder, Set, TransactionTrait,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

use crate::auth::CurrentUser;
use crate::error::{AppError, FieldError};
use crate::state::AppState;
use crate::support::audit::{AuditEntry, record};
use crate::support::time::now;

#[derive(Debug, Serialize)]
pub struct MineItem {
    pub id: Uuid,
    pub project_class: String,
    pub title: Option<String>,
    pub status: String,
    pub updated_at: String,
    pub submitted_at: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ApplicationView {
    pub id: Uuid,
    pub project_class: String,
    pub title: Option<String>,
    pub keywords: Option<String>,
    pub status: String,
    pub editable: bool,
    pub principal_name: Option<String>,
    pub gender: Option<String>,
    pub ethnicity: Option<String>,
    pub birth_date: Option<String>,
    pub admin_title: Option<String>,
    pub professional_title: Option<String>,
    pub research_expertise: Option<String>,
    pub education: Option<String>,
    pub degree: Option<String>,
    pub mobile_phone: Option<String>,
    pub department_name: Option<String>,
    pub filled_on: Option<String>,
    pub commitment_accepted: bool,
    pub members: Vec<MemberView>,
    pub recommender: Option<RecommenderView>,
    pub budget: BudgetView,
    pub ss_achievements: Vec<AchievementView>,
    pub ss_design: DesignView,
    pub ss_foundation: FoundationView,
    pub ss_stage_outputs: Vec<OutputView>,
    pub ss_final_outputs: Vec<OutputView>,
    pub ns_profile: Option<String>,
    pub ns_content: Option<String>,
    pub ns_indicators: IndicatorView,
    pub ns_schedules: Vec<ScheduleView>,
    pub ns_equipments: Vec<EquipmentView>,
}

#[derive(Debug, Serialize)]
pub struct MemberView {
    pub name: String,
    pub birth_date: Option<String>,
    pub professional_title: Option<String>,
    pub research_expertise: Option<String>,
    pub education: Option<String>,
    pub degree: Option<String>,
    pub organization: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct RecommenderView {
    pub name: String,
    pub professional_title: String,
    pub expertise: String,
    pub organization: String,
    pub opinion: Option<String>,
    pub signed_on: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct BudgetLineView {
    pub code: String,
    pub name: String,
    pub group_name: String,
    pub parent_code: Option<String>,
    pub fillable: bool,
    pub amount: String,
}

#[derive(Debug, Serialize)]
pub struct BudgetView {
    pub lines: Vec<BudgetLineView>,
    pub direct_total: String,
    pub indirect_total: String,
    pub total: String,
}

#[derive(Debug, Serialize)]
pub struct AchievementView {
    pub title: String,
    pub researchers: Option<String>,
    pub outcome_form: Option<String>,
    pub publication: Option<String>,
    pub published_on: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct DesignView {
    pub topic_basis: Option<String>,
    pub research_content: Option<String>,
    pub methodology: Option<String>,
    pub innovation: Option<String>,
    pub expected_outcome: Option<String>,
    pub reference_list: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct FoundationView {
    pub academic_resume: Option<String>,
    pub research_basis: Option<String>,
    pub undertaken_projects: Option<String>,
    pub support_conditions: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct OutputView {
    pub complete_on: Option<String>,
    pub title: String,
    pub outcome_form: Option<String>,
    pub owner_name: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct IndicatorView {
    pub paper_international: i32,
    pub paper_domestic: i32,
    pub paper_provincial: i32,
    pub paper_core: i32,
    pub paper_general: i32,
    pub report_tech: i32,
    pub report_research: i32,
    pub report_transfer: i32,
    pub report_other: i32,
    pub monograph: i32,
    pub patent_invention_applied: i32,
    pub patent_invention_granted: i32,
    pub patent_utility_applied: i32,
    pub patent_utility_granted: i32,
    pub patent_design_applied: i32,
    pub patent_design_granted: i32,
    pub patent_pct_applied: i32,
    pub patent_pct_granted: i32,
    pub patent_other_applied: i32,
    pub patent_other_granted: i32,
    pub software_copyright_applied: i32,
    pub software_copyright_registered: i32,
    pub ic_layout_applied: i32,
    pub ic_layout_registered: i32,
    pub plant_variety_applied: i32,
    pub plant_variety_registered: i32,
    pub standard_national_lead: i32,
    pub standard_national_participate: i32,
    pub standard_industry_lead: i32,
    pub standard_industry_participate: i32,
    pub standard_local_lead: i32,
    pub standard_local_participate: i32,
    pub standard_enterprise_lead: i32,
    pub standard_enterprise_participate: i32,
    pub pilot_line: i32,
    pub production_line: i32,
    pub new_sales_income: String,
    pub new_profit_tax: String,
    pub talent_train_senior: i32,
    pub talent_train_intermediate: i32,
    pub talent_introduce_senior: i32,
    pub new_product: i32,
    pub new_technology: i32,
    pub new_process: i32,
    pub new_material: i32,
    pub support_platform: i32,
    pub test_base: i32,
    pub demo_site: i32,
    pub benefit_note: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ScheduleView {
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub goal: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct EquipmentView {
    pub name_spec: String,
    pub quantity: i32,
    pub unit_price: String,
    pub amount: String,
    pub funding_source: Option<String>,
    pub purpose: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct SubjectView {
    pub code: String,
    pub name: String,
    pub group_name: String,
    pub parent_code: Option<String>,
    pub sort_order: i32,
    pub fillable: bool,
}

#[derive(Debug, Deserialize)]
pub struct CreateInput {
    pub project_class: String,
}

pub async fn create(state: &AppState, user: &CurrentUser, input: CreateInput) -> Result<ApplicationView, AppError> {
    if !user.has_role(UserRole::Teacher) {
        return Err(AppError::Forbidden("只有教师可以新建申报书".to_owned()));
    }
    let class = parse_class(&input.project_class)?;
    let cycle = open_cycle(&state.db).await?;
    let profile = users::Entity::find_by_id(user.id)
        .one(&state.db)
        .await?
        .ok_or_else(|| AppError::Unauthorized("登录已失效，请重新登录".to_owned()))?;
    let timestamp = now();
    let model = applications::ActiveModel {
        id: Set(Uuid::new_v4()),
        cycle_id: Set(cycle.id),
        project_class: Set(class),
        principal_id: Set(user.id),
        principal_name: Set(Some(profile.real_name)),
        gender: Set(profile.gender),
        ethnicity: Set(profile.ethnicity),
        birth_date: Set(profile.birth_date),
        admin_title: Set(profile.admin_title),
        professional_title: Set(profile.professional_title),
        research_expertise: Set(profile.research_expertise),
        education: Set(profile.education),
        degree: Set(profile.degree),
        mobile_phone: Set(profile.phone.clone()),
        contact_phone: Set(profile.phone),
        department_id: Set(profile.department_id),
        filled_on: Set(Some(timestamp.date_naive())),
        status: Set(ApplicationStatus::Draft),
        commitment_accepted: Set(false),
        created_at: Set(timestamp),
        updated_at: Set(timestamp),
        ..Default::default()
    };
    let saved = model.insert(&state.db).await?;
    write_audit(state, user.id, "application_created", saved.id).await;
    load_view(&state.db, saved.id).await
}

pub async fn mine(db: &DatabaseConnection, user: &CurrentUser) -> Result<Vec<MineItem>, AppError> {
    if !user.has_role(UserRole::Teacher) {
        return Err(AppError::Forbidden("只有教师可以查看本人申报书".to_owned()));
    }
    let rows = applications::Entity::find()
        .filter(applications::Column::PrincipalId.eq(user.id))
        .order_by_desc(applications::Column::UpdatedAt)
        .all(db)
        .await?;
    Ok(rows
        .into_iter()
        .map(|row| MineItem {
            id: row.id,
            project_class: class_code(&row.project_class).to_owned(),
            title: row.title,
            status: status_code(&row.status).to_owned(),
            updated_at: row.updated_at.to_rfc3339(),
            submitted_at: row.submitted_at.map(|value| value.to_rfc3339()),
        })
        .collect())
}

pub async fn detail(db: &DatabaseConnection, user: &CurrentUser, id: Uuid) -> Result<ApplicationView, AppError> {
    let app = load_owned(db, user, id).await?;
    let _ = app;
    load_view(db, id).await
}

pub async fn delete_draft(state: &AppState, user: &CurrentUser, id: Uuid) -> Result<(), AppError> {
    let result = applications::Entity::delete_many()
        .filter(applications::Column::Id.eq(id))
        .filter(applications::Column::PrincipalId.eq(user.id))
        .filter(applications::Column::Status.eq(ApplicationStatus::Draft))
        .exec(&state.db)
        .await?;
    if result.rows_affected == 0 {
        let exists = applications::Entity::find_by_id(id).one(&state.db).await?;
        return match exists {
            Some(app) if app.principal_id != user.id => {
                Err(AppError::Forbidden("只能删除本人的草稿".to_owned()))
            }
            Some(_) => Err(AppError::Conflict("只有未提交的草稿可以删除".to_owned())),
            None => Err(AppError::NotFound("申报书不存在".to_owned())),
        };
    }
    write_audit(state, user.id, "application_deleted", id).await;
    Ok(())
}

pub async fn save_section(
    state: &AppState,
    user: &CurrentUser,
    id: Uuid,
    section: &str,
    body: Value,
) -> Result<(), AppError> {
    let app = load_owned(&state.db, user, id).await?;
    ensure_editable(&app)?;
    ensure_section_class(&app, section)?;
    let txn = state.db.begin().await?;
    match section {
        "basic" => save_basic(&txn, &app, body).await?,
        "members" => save_members(&txn, id, body).await?,
        "recommender" => save_recommender(&txn, id, body).await?,
        "budget" => save_budget(&txn, &app, body).await?,
        "ss-achievements" => save_achievements(&txn, id, body).await?,
        "ss-design" => save_design(&txn, id, body).await?,
        "ss-foundation" => save_foundation(&txn, id, body).await?,
        "ss-outputs" => save_outputs(&txn, id, body).await?,
        "ns-profile" => save_profile(&txn, id, body).await?,
        "ns-content" => save_content(&txn, id, body).await?,
        "ns-indicators" => save_indicators(&txn, id, body).await?,
        "ns-schedules" => save_schedules(&txn, id, body).await?,
        "ns-equipments" => save_equipments(&txn, id, body).await?,
        _ => return Err(AppError::NotFound("没有这个申报栏目".to_owned())),
    }
    touch(&txn, id).await?;
    txn.commit().await?;
    Ok(())
}

pub async fn submit(state: &AppState, user: &CurrentUser, id: Uuid, accepted: bool) -> Result<ApplicationView, AppError> {
    if !accepted {
        return Err(invalid("commitment_accepted", "提交前必须确认申请者承诺"));
    }
    let app = load_owned(&state.db, user, id).await?;
    ensure_editable(&app)?;
    if !matches!(
        app.status,
        ApplicationStatus::Draft | ApplicationStatus::DeptReturned | ApplicationStatus::OfficeReturned
    ) {
        return Err(AppError::Conflict("当前状态不能提交".to_owned()));
    }
    validate_for_submit(&state.db, &app).await?;
    if app.status == ApplicationStatus::Draft {
        let _ = open_cycle(&state.db).await?;
    }
    let timestamp = now();
    let result = applications::Entity::update_many()
        .col_expr(applications::Column::Status, Expr::value(ApplicationStatus::Submitted.to_value()))
        .col_expr(applications::Column::CommitmentAccepted, Expr::value(true))
        .col_expr(applications::Column::CommitmentAcceptedAt, Expr::value(timestamp))
        .col_expr(applications::Column::SubmittedAt, Expr::value(timestamp))
        .col_expr(applications::Column::UpdatedAt, Expr::value(timestamp))
        .filter(applications::Column::Id.eq(id))
        .filter(applications::Column::PrincipalId.eq(user.id))
        .filter(applications::Column::Status.is_in([
            ApplicationStatus::Draft,
            ApplicationStatus::DeptReturned,
            ApplicationStatus::OfficeReturned,
        ]))
        .exec(&state.db)
        .await?;
    if result.rows_affected == 0 {
        return Err(AppError::Conflict("申报书状态已变化，请刷新后重试".to_owned()));
    }
    write_audit(state, user.id, "application_submitted", id).await;
    load_view(&state.db, id).await
}

pub async fn withdraw(state: &AppState, user: &CurrentUser, id: Uuid) -> Result<ApplicationView, AppError> {
    let app = load_owned(&state.db, user, id).await?;
    if app.status != ApplicationStatus::Submitted {
        return Err(AppError::Conflict("只有待系审核的申报书可以撤回".to_owned()));
    }
    let reviewed = reviews::Entity::find()
        .filter(reviews::Column::ApplicationId.eq(id))
        .filter(reviews::Column::Stage.eq(ReviewStage::Department))
        .one(&state.db)
        .await?;
    if reviewed.is_some() {
        return Err(AppError::Conflict("系审核已经开始，不能撤回".to_owned()));
    }
    let timestamp = now();
    let result = applications::Entity::update_many()
        .col_expr(applications::Column::Status, Expr::value(ApplicationStatus::Draft.to_value()))
        .col_expr(applications::Column::SubmittedAt, Expr::value(Option::<chrono::DateTime<chrono::FixedOffset>>::None))
        .col_expr(applications::Column::UpdatedAt, Expr::value(timestamp))
        .filter(applications::Column::Id.eq(id))
        .filter(applications::Column::Status.eq(ApplicationStatus::Submitted))
        .exec(&state.db)
        .await?;
    if result.rows_affected == 0 {
        return Err(AppError::Conflict("申报书状态已变化，请刷新后重试".to_owned()));
    }
    write_audit(state, user.id, "application_withdrawn", id).await;
    load_view(&state.db, id).await
}

pub async fn subjects(db: &DatabaseConnection, family: &str) -> Result<Vec<SubjectView>, AppError> {
    let family = parse_family(family)?;
    let rows = budget_subjects::Entity::find()
        .filter(budget_subjects::Column::AppliesTo.eq(family))
        .order_by_asc(budget_subjects::Column::SortOrder)
        .all(db)
        .await?;
    Ok(rows
        .into_iter()
        .map(|row| SubjectView {
            code: row.code,
            name: row.name,
            group_name: row.group_name,
            parent_code: row.parent_code,
            sort_order: row.sort_order,
            fillable: row.fillable,
        })
        .collect())
}

async fn save_basic(txn: &DatabaseTransaction, app: &applications::Model, body: Value) -> Result<(), AppError> {
    #[derive(Deserialize)]
    struct Input {
        title: Option<String>,
        keywords: Option<String>,
        filled_on: Option<String>,
        gender: Option<String>,
        ethnicity: Option<String>,
        birth_date: Option<String>,
        admin_title: Option<String>,
        professional_title: Option<String>,
        research_expertise: Option<String>,
        education: Option<String>,
        degree: Option<String>,
        mobile_phone: Option<String>,
    }
    let input: Input = decode(body)?;
    let mut issues = Vec::new();
    let title = clean(input.title);
    if let Some(title) = &title {
        if title.chars().count() > 40 {
            push_issue(&mut issues, "title", "项目名称最多 40 个字");
        }
    }
    if keyword_count(input.keywords.as_deref().unwrap_or("")) > 5 {
        push_issue(&mut issues, "keywords", "关键词最多 5 个，请用分号隔开");
    }
    let filled_on = take_date(&mut issues, input.filled_on.as_deref(), "filled_on", "填表日期");
    let birth_date = take_date(&mut issues, input.birth_date.as_deref(), "birth_date", "出生年月");
    let gender = take_text(&mut issues, input.gender, 8, "gender", "性别最多 8 个字");
    let ethnicity = take_text(&mut issues, input.ethnicity, 32, "ethnicity", "民族最多 32 个字");
    let admin_title = take_text(&mut issues, input.admin_title, 64, "admin_title", "行政职务最多 64 个字");
    let professional_title = take_text(&mut issues, input.professional_title, 64, "professional_title", "专业职务最多 64 个字");
    let research_expertise = take_text(&mut issues, input.research_expertise, 120, "research_expertise", "研究专长最多 120 个字");
    let education = take_text(&mut issues, input.education, 32, "education", "最后学历最多 32 个字");
    let degree = take_text(&mut issues, input.degree, 32, "degree", "最后学位最多 32 个字");
    let phone = take_text(&mut issues, input.mobile_phone, 20, "mobile_phone", "手机号码最多 20 位");
    if !issues.is_empty() {
        return Err(AppError::Validation(issues));
    }
    let mut active: applications::ActiveModel = app.clone().into();
    active.title = Set(title);
    active.keywords = Set(clean(input.keywords));
    active.filled_on = Set(filled_on);
    active.gender = Set(gender);
    active.ethnicity = Set(ethnicity);
    active.birth_date = Set(birth_date);
    active.admin_title = Set(admin_title);
    active.professional_title = Set(professional_title);
    active.research_expertise = Set(research_expertise);
    active.education = Set(education);
    active.degree = Set(degree);
    active.mobile_phone = Set(phone.clone());
    active.contact_phone = Set(phone);
    active.updated_at = Set(now());
    active.update(txn).await?;
    Ok(())
}

async fn save_members(txn: &DatabaseTransaction, id: Uuid, body: Value) -> Result<(), AppError> {
    #[derive(Deserialize)]
    struct Input {
        members: Vec<MemberBody>,
    }
    #[derive(Deserialize)]
    struct MemberBody {
        name: String,
        birth_date: Option<String>,
        professional_title: Option<String>,
        research_expertise: Option<String>,
        education: Option<String>,
        degree: Option<String>,
        organization: Option<String>,
    }
    let input: Input = decode(body)?;
    let mut issues = Vec::new();
    let mut ready = Vec::new();
    for (index, row) in input.members.into_iter().enumerate() {
        let blank = row.name.trim().is_empty()
            && row.birth_date.as_deref().unwrap_or("").trim().is_empty()
            && row.professional_title.as_deref().unwrap_or("").trim().is_empty()
            && row.research_expertise.as_deref().unwrap_or("").trim().is_empty()
            && row.education.as_deref().unwrap_or("").trim().is_empty()
            && row.degree.as_deref().unwrap_or("").trim().is_empty()
            && row.organization.as_deref().unwrap_or("").trim().is_empty();
        if blank {
            continue;
        }
        let label = member_label(index, &row.name);
        if row.name.trim().is_empty() {
            push_issue(&mut issues, &format!("members.{index}.name"), &format!("{label}写了其他资料，但没有填写姓名"));
            continue;
        }
        let name = take_required(&mut issues, &row.name, 64, &format!("members.{index}.name"), &format!("{label}的姓名最多 64 个字"));
        let birth_date = take_date(&mut issues, row.birth_date.as_deref(), &format!("members.{index}.birth_date"), &format!("{label}的出生年月"));
        let professional_title = take_text(&mut issues, row.professional_title, 64, &format!("members.{index}.professional_title"), &format!("{label}的专业职务最多 64 个字"));
        let research_expertise = take_text(&mut issues, row.research_expertise, 120, &format!("members.{index}.research_expertise"), &format!("{label}的研究专长最多 120 个字"));
        let education = take_text(&mut issues, row.education, 32, &format!("members.{index}.education"), &format!("{label}的学历最多 32 个字"));
        let degree = take_text(&mut issues, row.degree, 32, &format!("members.{index}.degree"), &format!("{label}的学位最多 32 个字"));
        let organization = take_text(&mut issues, row.organization, 120, &format!("members.{index}.organization"), &format!("{label}的工作单位最多 120 个字"));
        ready.push((name, birth_date, professional_title, research_expertise, education, degree, organization));
    }
    if ready.len() > 5 {
        push_issue(&mut issues, "members", "主要参加者最多 5 人，不含项目负责人");
    }
    if !issues.is_empty() {
        return Err(AppError::Validation(issues));
    }
    application_members::Entity::delete_many()
        .filter(application_members::Column::ApplicationId.eq(id))
        .exec(txn)
        .await?;
    for (index, (name, birth_date, professional_title, research_expertise, education, degree, organization)) in ready.into_iter().enumerate() {
        let Some(name) = name else {
            continue;
        };
        let sort_order = i16::try_from(index + 1).unwrap_or(1);
        application_members::ActiveModel {
            id: Set(Uuid::new_v4()),
            application_id: Set(id),
            sort_order: Set(sort_order),
            name: Set(name),
            birth_date: Set(birth_date),
            professional_title: Set(professional_title),
            research_expertise: Set(research_expertise),
            education: Set(education),
            degree: Set(degree),
            organization: Set(organization),
            user_id: Set(None),
            created_at: Set(now()),
            updated_at: Set(now()),
        }
        .insert(txn)
        .await?;
    }
    Ok(())
}

async fn save_recommender(txn: &DatabaseTransaction, id: Uuid, body: Value) -> Result<(), AppError> {
    #[derive(Deserialize)]
    struct Input {
        name: Option<String>,
        professional_title: Option<String>,
        expertise: Option<String>,
        organization: Option<String>,
        opinion: Option<String>,
        signed_on: Option<String>,
    }
    let input: Input = decode(body)?;
    recommenders::Entity::delete_by_id(id).exec(txn).await?;
    let Some(name) = clean(input.name) else {
        return Ok(());
    };
    let title = clean(input.professional_title).ok_or_else(|| invalid("professional_title", "请填写推荐人专业技术职务"))?;
    let expertise = clean(input.expertise).ok_or_else(|| invalid("expertise", "请填写推荐人研究专长"))?;
    let organization = clean(input.organization).ok_or_else(|| invalid("organization", "请填写推荐人工作单位"))?;
    recommenders::ActiveModel {
        application_id: Set(id),
        name: Set(require_len(&name, 64, "name", "推荐人姓名过长")?),
        professional_title: Set(require_len(&title, 64, "professional_title", "专业技术职务过长")?),
        expertise: Set(require_len(&expertise, 120, "expertise", "研究专长过长")?),
        organization: Set(require_len(&organization, 120, "organization", "工作单位过长")?),
        opinion: Set(clean(input.opinion)),
        signed_on: Set(parse_date(input.signed_on.as_deref(), "signed_on")?),
        created_at: Set(now()),
        updated_at: Set(now()),
    }
    .insert(txn)
    .await?;
    Ok(())
}

async fn save_budget(txn: &DatabaseTransaction, app: &applications::Model, body: Value) -> Result<(), AppError> {
    #[derive(Deserialize)]
    struct Input {
        lines: Vec<BudgetBody>,
    }
    #[derive(Deserialize)]
    struct BudgetBody {
        subject_code: String,
        amount: String,
    }
    let input: Input = decode(body)?;
    let family = family_of(&app.project_class);
    let subjects = budget_subjects::Entity::find()
        .filter(budget_subjects::Column::AppliesTo.eq(family.clone()))
        .all(txn)
        .await?;
    application_budgets::Entity::delete_many()
        .filter(application_budgets::Column::ApplicationId.eq(app.id))
        .exec(txn)
        .await?;
    for line in input.lines {
        let subject = subjects.iter().find(|item| item.code == line.subject_code);
        let Some(subject) = subject else {
            return Err(invalid("budget", "经费科目与申报类别不匹配"));
        };
        if !subject.fillable {
            return Err(invalid("budget", "合计行由系统计算，不能直接填写"));
        }
        let amount = match read_money(&line.amount) {
            Ok(amount) => amount,
            Err(reason) => {
                return Err(invalid(
                    &format!("budget.{}", subject.code),
                    &format!("经费科目「{}」的金额{reason}", subject.name),
                ));
            }
        };
        application_budgets::ActiveModel {
            id: Set(Uuid::new_v4()),
            application_id: Set(app.id),
            subject_code: Set(subject.code.clone()),
            amount: Set(amount),
            created_at: Set(now()),
            updated_at: Set(now()),
        }
        .insert(txn)
        .await?;
    }
    Ok(())
}

async fn save_achievements(txn: &DatabaseTransaction, id: Uuid, body: Value) -> Result<(), AppError> {
    #[derive(Deserialize)]
    struct Input {
        rows: Vec<AchievementBody>,
    }
    #[derive(Deserialize)]
    struct AchievementBody {
        title: String,
        researchers: Option<String>,
        outcome_form: Option<String>,
        publication: Option<String>,
        published_on: Option<String>,
    }
    let input: Input = decode(body)?;
    let rows: Vec<_> = input.rows.into_iter().filter(|row| !row.title.trim().is_empty()).collect();
    ss_prior_achievements::Entity::delete_many()
        .filter(ss_prior_achievements::Column::ApplicationId.eq(id))
        .exec(txn)
        .await?;
    for (index, row) in rows.iter().enumerate() {
        ss_prior_achievements::ActiveModel {
            id: Set(Uuid::new_v4()),
            application_id: Set(id),
            sort_order: Set(sort_of(index)?),
            title: Set(require_len(&row.title, 200, "title", "成果名称过长")?),
            researchers: Set(limited(row.researchers.clone(), 120, "researchers", "研究者过长")?),
            outcome_form: Set(limited(row.outcome_form.clone(), 64, "outcome_form", "成果形式过长")?),
            publication: Set(clean(row.publication.clone())),
            published_on: Set(limited(row.published_on.clone(), 32, "published_on", "发表时间过长")?),
            created_at: Set(now()),
            updated_at: Set(now()),
        }
        .insert(txn)
        .await?;
    }
    Ok(())
}

async fn save_design(txn: &DatabaseTransaction, id: Uuid, body: Value) -> Result<(), AppError> {
    let input: DesignView = decode(body)?;
    ss_designs::Entity::delete_by_id(id).exec(txn).await?;
    ss_designs::ActiveModel {
        application_id: Set(id),
        topic_basis: Set(clean(input.topic_basis)),
        research_content: Set(clean(input.research_content)),
        methodology: Set(clean(input.methodology)),
        innovation: Set(clean(input.innovation)),
        expected_outcome: Set(clean(input.expected_outcome)),
        reference_list: Set(clean(input.reference_list)),
        created_at: Set(now()),
        updated_at: Set(now()),
    }
    .insert(txn)
    .await?;
    Ok(())
}

async fn save_foundation(txn: &DatabaseTransaction, id: Uuid, body: Value) -> Result<(), AppError> {
    let input: FoundationView = decode(body)?;
    ss_foundations::Entity::delete_by_id(id).exec(txn).await?;
    ss_foundations::ActiveModel {
        application_id: Set(id),
        academic_resume: Set(clean(input.academic_resume)),
        research_basis: Set(clean(input.research_basis)),
        undertaken_projects: Set(clean(input.undertaken_projects)),
        support_conditions: Set(clean(input.support_conditions)),
        created_at: Set(now()),
        updated_at: Set(now()),
    }
    .insert(txn)
    .await?;
    Ok(())
}

async fn save_outputs(txn: &DatabaseTransaction, id: Uuid, body: Value) -> Result<(), AppError> {
    #[derive(Deserialize)]
    struct Input {
        stage: Vec<OutputBody>,
        #[serde(rename = "final")]
        finals: Vec<OutputBody>,
    }
    let input: Input = decode(body)?;
    let stage: Vec<_> = input.stage.into_iter().filter(|row| !row.title.trim().is_empty()).collect();
    let finals: Vec<_> = input.finals.into_iter().filter(|row| !row.title.trim().is_empty()).collect();
    if stage.len() > 10 {
        return Err(invalid("stage", "阶段性成果最多 10 项"));
    }
    if finals.len() > 2 {
        return Err(invalid("final", "最终成果最多 2 项"));
    }
    ss_expected_outputs::Entity::delete_many()
        .filter(ss_expected_outputs::Column::ApplicationId.eq(id))
        .exec(txn)
        .await?;
    write_outputs(txn, id, OutputKind::Stage, &stage).await?;
    write_outputs(txn, id, OutputKind::Final, &finals).await?;
    Ok(())
}

async fn write_outputs(
    txn: &DatabaseTransaction,
    id: Uuid,
    kind: OutputKind,
    rows: &[OutputBody],
) -> Result<(), AppError> {
    for (index, row) in rows.iter().enumerate() {
        ss_expected_outputs::ActiveModel {
            id: Set(Uuid::new_v4()),
            application_id: Set(id),
            kind: Set(kind.clone()),
            sort_order: Set(sort_of(index)?),
            complete_on: Set(limited(row.complete_on.clone(), 32, "complete_on", "完成时间过长")?),
            title: Set(require_len(&row.title, 200, "title", "成果名称过长")?),
            outcome_form: Set(limited(row.outcome_form.clone(), 64, "outcome_form", "成果形式过长")?),
            owner_name: Set(limited(row.owner_name.clone(), 64, "owner_name", "负责人姓名过长")?),
            created_at: Set(now()),
            updated_at: Set(now()),
        }
        .insert(txn)
        .await?;
    }
    Ok(())
}

#[derive(Deserialize)]
struct OutputBody {
    complete_on: Option<String>,
    title: String,
    outcome_form: Option<String>,
    owner_name: Option<String>,
}

async fn save_profile(txn: &DatabaseTransaction, id: Uuid, body: Value) -> Result<(), AppError> {
    #[derive(Deserialize)]
    struct Input {
        summary: Option<String>,
    }
    let input: Input = decode(body)?;
    ns_profiles::Entity::delete_by_id(id).exec(txn).await?;
    ns_profiles::ActiveModel {
        application_id: Set(id),
        summary: Set(clean(input.summary)),
        created_at: Set(now()),
        updated_at: Set(now()),
    }
    .insert(txn)
    .await?;
    Ok(())
}

async fn save_content(txn: &DatabaseTransaction, id: Uuid, body: Value) -> Result<(), AppError> {
    #[derive(Deserialize)]
    struct Input {
        content: Option<String>,
    }
    let input: Input = decode(body)?;
    ns_research_contents::Entity::delete_by_id(id).exec(txn).await?;
    ns_research_contents::ActiveModel {
        application_id: Set(id),
        content: Set(clean(input.content)),
        created_at: Set(now()),
        updated_at: Set(now()),
    }
    .insert(txn)
    .await?;
    Ok(())
}

async fn save_indicators(txn: &DatabaseTransaction, id: Uuid, body: Value) -> Result<(), AppError> {
    #[derive(Deserialize, Default)]
    struct Input {
        #[serde(default)]
        paper_international: i32,
        #[serde(default)]
        paper_domestic: i32,
        #[serde(default)]
        paper_provincial: i32,
        #[serde(default)]
        paper_core: i32,
        #[serde(default)]
        paper_general: i32,
        #[serde(default)]
        report_tech: i32,
        #[serde(default)]
        report_research: i32,
        #[serde(default)]
        report_transfer: i32,
        #[serde(default)]
        report_other: i32,
        #[serde(default)]
        monograph: i32,
        #[serde(default)]
        patent_invention_applied: i32,
        #[serde(default)]
        patent_invention_granted: i32,
        #[serde(default)]
        patent_utility_applied: i32,
        #[serde(default)]
        patent_utility_granted: i32,
        #[serde(default)]
        patent_design_applied: i32,
        #[serde(default)]
        patent_design_granted: i32,
        #[serde(default)]
        patent_pct_applied: i32,
        #[serde(default)]
        patent_pct_granted: i32,
        #[serde(default)]
        patent_other_applied: i32,
        #[serde(default)]
        patent_other_granted: i32,
        #[serde(default)]
        software_copyright_applied: i32,
        #[serde(default)]
        software_copyright_registered: i32,
        #[serde(default)]
        ic_layout_applied: i32,
        #[serde(default)]
        ic_layout_registered: i32,
        #[serde(default)]
        plant_variety_applied: i32,
        #[serde(default)]
        plant_variety_registered: i32,
        #[serde(default)]
        standard_national_lead: i32,
        #[serde(default)]
        standard_national_participate: i32,
        #[serde(default)]
        standard_industry_lead: i32,
        #[serde(default)]
        standard_industry_participate: i32,
        #[serde(default)]
        standard_local_lead: i32,
        #[serde(default)]
        standard_local_participate: i32,
        #[serde(default)]
        standard_enterprise_lead: i32,
        #[serde(default)]
        standard_enterprise_participate: i32,
        #[serde(default)]
        pilot_line: i32,
        #[serde(default)]
        production_line: i32,
        #[serde(default)]
        new_sales_income: String,
        #[serde(default)]
        new_profit_tax: String,
        #[serde(default)]
        talent_train_senior: i32,
        #[serde(default)]
        talent_train_intermediate: i32,
        #[serde(default)]
        talent_introduce_senior: i32,
        #[serde(default)]
        new_product: i32,
        #[serde(default)]
        new_technology: i32,
        #[serde(default)]
        new_process: i32,
        #[serde(default)]
        new_material: i32,
        #[serde(default)]
        support_platform: i32,
        #[serde(default)]
        test_base: i32,
        #[serde(default)]
        demo_site: i32,
        benefit_note: Option<String>,
    }
    let input: Input = decode(body)?;
    let counts = [
        input.paper_international,
        input.paper_domestic,
        input.paper_provincial,
        input.paper_core,
        input.paper_general,
        input.report_tech,
        input.report_research,
        input.report_transfer,
        input.report_other,
        input.monograph,
        input.patent_invention_applied,
        input.patent_invention_granted,
        input.patent_utility_applied,
        input.patent_utility_granted,
        input.patent_design_applied,
        input.patent_design_granted,
        input.patent_pct_applied,
        input.patent_pct_granted,
        input.patent_other_applied,
        input.patent_other_granted,
        input.software_copyright_applied,
        input.software_copyright_registered,
        input.ic_layout_applied,
        input.ic_layout_registered,
        input.plant_variety_applied,
        input.plant_variety_registered,
        input.standard_national_lead,
        input.standard_national_participate,
        input.standard_industry_lead,
        input.standard_industry_participate,
        input.standard_local_lead,
        input.standard_local_participate,
        input.standard_enterprise_lead,
        input.standard_enterprise_participate,
        input.pilot_line,
        input.production_line,
        input.talent_train_senior,
        input.talent_train_intermediate,
        input.talent_introduce_senior,
        input.new_product,
        input.new_technology,
        input.new_process,
        input.new_material,
        input.support_platform,
        input.test_base,
        input.demo_site,
    ];
    if counts.iter().any(|value| *value < 0) {
        return Err(invalid("indicators", "指标数量不能为负"));
    }
    ns_indicators::Entity::delete_by_id(id).exec(txn).await?;
    ns_indicators::ActiveModel {
        application_id: Set(id),
        paper_international: Set(input.paper_international),
        paper_domestic: Set(input.paper_domestic),
        paper_provincial: Set(input.paper_provincial),
        paper_core: Set(input.paper_core),
        paper_general: Set(input.paper_general),
        report_tech: Set(input.report_tech),
        report_research: Set(input.report_research),
        report_transfer: Set(input.report_transfer),
        report_other: Set(input.report_other),
        monograph: Set(input.monograph),
        patent_invention_applied: Set(input.patent_invention_applied),
        patent_invention_granted: Set(input.patent_invention_granted),
        patent_utility_applied: Set(input.patent_utility_applied),
        patent_utility_granted: Set(input.patent_utility_granted),
        patent_design_applied: Set(input.patent_design_applied),
        patent_design_granted: Set(input.patent_design_granted),
        patent_pct_applied: Set(input.patent_pct_applied),
        patent_pct_granted: Set(input.patent_pct_granted),
        patent_other_applied: Set(input.patent_other_applied),
        patent_other_granted: Set(input.patent_other_granted),
        software_copyright_applied: Set(input.software_copyright_applied),
        software_copyright_registered: Set(input.software_copyright_registered),
        ic_layout_applied: Set(input.ic_layout_applied),
        ic_layout_registered: Set(input.ic_layout_registered),
        plant_variety_applied: Set(input.plant_variety_applied),
        plant_variety_registered: Set(input.plant_variety_registered),
        standard_national_lead: Set(input.standard_national_lead),
        standard_national_participate: Set(input.standard_national_participate),
        standard_industry_lead: Set(input.standard_industry_lead),
        standard_industry_participate: Set(input.standard_industry_participate),
        standard_local_lead: Set(input.standard_local_lead),
        standard_local_participate: Set(input.standard_local_participate),
        standard_enterprise_lead: Set(input.standard_enterprise_lead),
        standard_enterprise_participate: Set(input.standard_enterprise_participate),
        pilot_line: Set(input.pilot_line),
        production_line: Set(input.production_line),
        new_sales_income: Set(money(if input.new_sales_income.is_empty() { "0" } else { &input.new_sales_income }, "new_sales_income")?),
        new_profit_tax: Set(money(if input.new_profit_tax.is_empty() { "0" } else { &input.new_profit_tax }, "new_profit_tax")?),
        talent_train_senior: Set(input.talent_train_senior),
        talent_train_intermediate: Set(input.talent_train_intermediate),
        talent_introduce_senior: Set(input.talent_introduce_senior),
        new_product: Set(input.new_product),
        new_technology: Set(input.new_technology),
        new_process: Set(input.new_process),
        new_material: Set(input.new_material),
        support_platform: Set(input.support_platform),
        test_base: Set(input.test_base),
        demo_site: Set(input.demo_site),
        benefit_note: Set(clean(input.benefit_note)),
        created_at: Set(now()),
        updated_at: Set(now()),
    }
    .insert(txn)
    .await?;
    Ok(())
}

async fn save_schedules(txn: &DatabaseTransaction, id: Uuid, body: Value) -> Result<(), AppError> {
    #[derive(Deserialize)]
    struct Input {
        rows: Vec<ScheduleBody>,
    }
    #[derive(Deserialize)]
    struct ScheduleBody {
        start_date: Option<String>,
        end_date: Option<String>,
        goal: Option<String>,
    }
    let input: Input = decode(body)?;
    ns_schedules::Entity::delete_many()
        .filter(ns_schedules::Column::ApplicationId.eq(id))
        .exec(txn)
        .await?;
    let mut sort = 1_i16;
    for (index, row) in input.rows.into_iter().enumerate() {
        let start = take_row_date(row.start_date.as_deref(), index, "start_date", "开始日期")?;
        let end = take_row_date(row.end_date.as_deref(), index, "end_date", "结束日期")?;
        let goal = clean(row.goal);
        if start.is_none() && end.is_none() && goal.is_none() {
            continue;
        }
        if let (Some(start), Some(end)) = (start, end) {
            if end < start {
                return Err(invalid(
                    &format!("schedules.{index}.end_date"),
                    &format!("第 {} 条进度的结束日期不能早于开始日期", index + 1),
                ));
            }
        }
        ns_schedules::ActiveModel {
            id: Set(Uuid::new_v4()),
            application_id: Set(id),
            sort_order: Set(sort),
            start_date: Set(start),
            end_date: Set(end),
            goal: Set(goal),
            created_at: Set(now()),
            updated_at: Set(now()),
        }
        .insert(txn)
        .await?;
        sort += 1;
    }
    Ok(())
}

async fn save_equipments(txn: &DatabaseTransaction, id: Uuid, body: Value) -> Result<(), AppError> {
    #[derive(Deserialize)]
    struct Input {
        rows: Vec<EquipmentBody>,
    }
    #[derive(Deserialize)]
    struct EquipmentBody {
        name_spec: String,
        quantity: i32,
        unit_price: String,
        funding_source: Option<String>,
        purpose: Option<String>,
    }
    let input: Input = decode(body)?;
    let rows: Vec<_> = input.rows.into_iter().filter(|row| !row.name_spec.trim().is_empty()).collect();
    ns_equipments::Entity::delete_many()
        .filter(ns_equipments::Column::ApplicationId.eq(id))
        .exec(txn)
        .await?;
    for (index, row) in rows.iter().enumerate() {
        if row.quantity <= 0 {
            return Err(invalid(
                &format!("equipments.{index}.quantity"),
                &format!("第 {} 台设备「{}」的数量必须大于 0", index + 1, row.name_spec.trim()),
            ));
        }
        let price = match read_money(&row.unit_price) {
            Ok(price) => price,
            Err(reason) => {
                return Err(invalid(
                    &format!("equipments.{index}.unit_price"),
                    &format!("第 {} 台设备「{}」的单价{reason}", index + 1, row.name_spec.trim()),
                ));
            }
        };
        if price < sea_orm::prelude::Decimal::from(5000) {
            return Err(invalid(
                &format!("equipments.{index}.unit_price"),
                &format!("第 {} 台设备「{}」的单价低于 5000 元，不列入清单", index + 1, row.name_spec.trim()),
            ));
        }
        let amount = price * sea_orm::prelude::Decimal::from(row.quantity);
        ns_equipments::ActiveModel {
            id: Set(Uuid::new_v4()),
            application_id: Set(id),
            sort_order: Set(sort_of(index)?),
            name_spec: Set(require_len(&row.name_spec, 200, "name_spec", "设备名称过长")?),
            quantity: Set(Some(row.quantity)),
            unit_price: Set(Some(price)),
            amount: Set(Some(amount)),
            funding_source: Set(limited(row.funding_source.clone(), 64, "funding_source", "资金来源过长")?),
            purpose: Set(clean(row.purpose.clone())),
            created_at: Set(now()),
            updated_at: Set(now()),
        }
        .insert(txn)
        .await?;
    }
    Ok(())
}

async fn validate_for_submit(db: &DatabaseConnection, app: &applications::Model) -> Result<(), AppError> {
    let title = app.title.as_deref().unwrap_or("").trim();
    if title.is_empty() || title.chars().count() > 40 {
        return Err(invalid("title", "请填写项目名称，且不超过 40 个字"));
    }
    if app.principal_name.as_deref().unwrap_or("").trim().is_empty() {
        return Err(invalid("principal_name", "请填写项目负责人"));
    }
    if app.department_id.is_none() {
        return Err(invalid("department_id", "负责人还没有所在部门"));
    }
    if keyword_count(app.keywords.as_deref().unwrap_or("")) > 5 {
        return Err(invalid("keywords", "关键词最多 5 个"));
    }
    let members = application_members::Entity::find()
        .filter(application_members::Column::ApplicationId.eq(app.id))
        .all(db)
        .await?;
    let recommender = recommenders::Entity::find_by_id(app.id).one(db).await?;
    if needs_recommender(
        app.professional_title.as_deref().unwrap_or(""),
        app.education.as_deref().unwrap_or(""),
        app.degree.as_deref().unwrap_or(""),
    ) && recommender.is_none()
    {
        return Err(invalid("recommender", "负责人不具备中级及以上职称，且不具备研究生及以上学历时，必须填写推荐人"));
    }
    if let Some(recommender) = &recommender {
        let same_as_principal = recommender.name == app.principal_name.as_deref().unwrap_or("");
        let same_as_member = members.iter().any(|member| member.name == recommender.name);
        if same_as_principal || same_as_member {
            return Err(invalid("recommender", "推荐人不能是项目负责人或参加者"));
        }
    }
    let budgets = application_budgets::Entity::find()
        .filter(application_budgets::Column::ApplicationId.eq(app.id))
        .all(db)
        .await?;
    let subjects = budget_subjects::Entity::find()
        .filter(budget_subjects::Column::AppliesTo.eq(family_of(&app.project_class)))
        .all(db)
        .await?;
    let total = budget_total(&subjects, &budgets);
    if total <= sea_orm::prelude::Decimal::ZERO {
        return Err(invalid("budget", "经费合计必须大于 0"));
    }
    Ok(())
}

fn budget_total(subjects: &[budget_subjects::Model], lines: &[application_budgets::Model]) -> sea_orm::prelude::Decimal {
    let mut direct = sea_orm::prelude::Decimal::ZERO;
    let mut indirect = sea_orm::prelude::Decimal::ZERO;
    for line in lines {
        let Some(subject) = subjects.iter().find(|item| item.code == line.subject_code) else {
            continue;
        };
        if subject.group_name == "direct" && subject.fillable {
            direct += line.amount;
        }
        if subject.group_name == "indirect" && subject.parent_code.is_none() {
            indirect += line.amount;
        }
    }
    direct + indirect
}

async fn load_view(db: &DatabaseConnection, id: Uuid) -> Result<ApplicationView, AppError> {
    let app = applications::Entity::find_by_id(id)
        .one(db)
        .await?
        .ok_or_else(|| AppError::NotFound("申报书不存在".to_owned()))?;
    let department_name = match app.department_id {
        Some(department_id) => departments::Entity::find_by_id(department_id)
            .one(db)
            .await?
            .map(|department| department.name),
        None => None,
    };
    let members = application_members::Entity::find()
        .filter(application_members::Column::ApplicationId.eq(id))
        .order_by_asc(application_members::Column::SortOrder)
        .all(db)
        .await?;
    let recommender = recommenders::Entity::find_by_id(id).one(db).await?;
    let family = family_of(&app.project_class);
    let subjects = budget_subjects::Entity::find()
        .filter(budget_subjects::Column::AppliesTo.eq(family))
        .order_by_asc(budget_subjects::Column::SortOrder)
        .all(db)
        .await?;
    let budgets = application_budgets::Entity::find()
        .filter(application_budgets::Column::ApplicationId.eq(id))
        .all(db)
        .await?;
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
    let direct = budget_sum(&subjects, &budgets, "direct", true);
    let indirect = budget_sum(&subjects, &budgets, "indirect", false);
    Ok(ApplicationView {
        id: app.id,
        project_class: class_code(&app.project_class).to_owned(),
        title: app.title,
        keywords: app.keywords,
        status: status_code(&app.status).to_owned(),
        editable: matches!(
            app.status,
            ApplicationStatus::Draft | ApplicationStatus::DeptReturned | ApplicationStatus::OfficeReturned
        ),
        principal_name: app.principal_name,
        gender: app.gender,
        ethnicity: app.ethnicity,
        birth_date: app.birth_date.map(|date| date.format("%Y-%m").to_string()),
        admin_title: app.admin_title,
        professional_title: app.professional_title,
        research_expertise: app.research_expertise,
        education: app.education,
        degree: app.degree,
        mobile_phone: app.mobile_phone,
        department_name,
        filled_on: app.filled_on.map(|date| date.format("%Y-%m-%d").to_string()),
        commitment_accepted: app.commitment_accepted,
        members: members
            .into_iter()
            .map(|row| MemberView {
                name: row.name,
                birth_date: row.birth_date.map(|date| date.format("%Y-%m").to_string()),
                professional_title: row.professional_title,
                research_expertise: row.research_expertise,
                education: row.education,
                degree: row.degree,
                organization: row.organization,
            })
            .collect(),
        recommender: recommender.map(|row| RecommenderView {
            name: row.name,
            professional_title: row.professional_title,
            expertise: row.expertise,
            organization: row.organization,
            opinion: row.opinion,
            signed_on: row.signed_on.map(|date| date.format("%Y-%m-%d").to_string()),
        }),
        budget: BudgetView {
            lines: subjects
                .iter()
                .map(|subject| BudgetLineView {
                    amount: budgets
                        .iter()
                        .find(|line| line.subject_code == subject.code)
                        .map(|line| money_text(line.amount))
                        .unwrap_or_else(|| "0.00".to_owned()),
                    code: subject.code.clone(),
                    name: subject.name.clone(),
                    group_name: subject.group_name.clone(),
                    parent_code: subject.parent_code.clone(),
                    fillable: subject.fillable,
                })
                .collect(),
            direct_total: money_text(direct),
            indirect_total: money_text(indirect),
            total: money_text(direct + indirect),
        },
        ss_achievements: achievements
            .into_iter()
            .map(|row| AchievementView {
                title: row.title,
                researchers: row.researchers,
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
            .map(|row| FoundationView {
                academic_resume: row.academic_resume,
                research_basis: row.research_basis,
                undertaken_projects: row.undertaken_projects,
                support_conditions: row.support_conditions,
            })
            .unwrap_or_default(),
        ss_stage_outputs: outputs
            .iter()
            .filter(|row| row.kind == OutputKind::Stage)
            .map(output_view)
            .collect(),
        ss_final_outputs: outputs
            .iter()
            .filter(|row| row.kind == OutputKind::Final)
            .map(output_view)
            .collect(),
        ns_profile: profile.and_then(|row| row.summary),
        ns_content: content.and_then(|row| row.content),
        ns_indicators: indicators.map(indicator_view).unwrap_or_else(empty_indicators),
        ns_schedules: schedules
            .into_iter()
            .map(|row| ScheduleView {
                start_date: row.start_date.map(|date| date.format("%Y-%m-%d").to_string()),
                end_date: row.end_date.map(|date| date.format("%Y-%m-%d").to_string()),
                goal: row.goal,
            })
            .collect(),
        ns_equipments: equipments
            .into_iter()
            .map(|row| EquipmentView {
                name_spec: row.name_spec,
                quantity: row.quantity.unwrap_or(0),
                unit_price: row.unit_price.map(money_text).unwrap_or_else(|| "0.00".to_owned()),
                amount: row.amount.map(money_text).unwrap_or_else(|| "0.00".to_owned()),
                funding_source: row.funding_source,
                purpose: row.purpose,
            })
            .collect(),
    })
}

fn output_view(row: &ss_expected_outputs::Model) -> OutputView {
    OutputView {
        complete_on: row.complete_on.clone(),
        title: row.title.clone(),
        outcome_form: row.outcome_form.clone(),
        owner_name: row.owner_name.clone(),
    }
}

fn indicator_view(row: ns_indicators::Model) -> IndicatorView {
    IndicatorView {
        paper_international: row.paper_international,
        paper_domestic: row.paper_domestic,
        paper_provincial: row.paper_provincial,
        paper_core: row.paper_core,
        paper_general: row.paper_general,
        report_tech: row.report_tech,
        report_research: row.report_research,
        report_transfer: row.report_transfer,
        report_other: row.report_other,
        monograph: row.monograph,
        patent_invention_applied: row.patent_invention_applied,
        patent_invention_granted: row.patent_invention_granted,
        patent_utility_applied: row.patent_utility_applied,
        patent_utility_granted: row.patent_utility_granted,
        patent_design_applied: row.patent_design_applied,
        patent_design_granted: row.patent_design_granted,
        patent_pct_applied: row.patent_pct_applied,
        patent_pct_granted: row.patent_pct_granted,
        patent_other_applied: row.patent_other_applied,
        patent_other_granted: row.patent_other_granted,
        software_copyright_applied: row.software_copyright_applied,
        software_copyright_registered: row.software_copyright_registered,
        ic_layout_applied: row.ic_layout_applied,
        ic_layout_registered: row.ic_layout_registered,
        plant_variety_applied: row.plant_variety_applied,
        plant_variety_registered: row.plant_variety_registered,
        standard_national_lead: row.standard_national_lead,
        standard_national_participate: row.standard_national_participate,
        standard_industry_lead: row.standard_industry_lead,
        standard_industry_participate: row.standard_industry_participate,
        standard_local_lead: row.standard_local_lead,
        standard_local_participate: row.standard_local_participate,
        standard_enterprise_lead: row.standard_enterprise_lead,
        standard_enterprise_participate: row.standard_enterprise_participate,
        pilot_line: row.pilot_line,
        production_line: row.production_line,
        new_sales_income: money_text(row.new_sales_income),
        new_profit_tax: money_text(row.new_profit_tax),
        talent_train_senior: row.talent_train_senior,
        talent_train_intermediate: row.talent_train_intermediate,
        talent_introduce_senior: row.talent_introduce_senior,
        new_product: row.new_product,
        new_technology: row.new_technology,
        new_process: row.new_process,
        new_material: row.new_material,
        support_platform: row.support_platform,
        test_base: row.test_base,
        demo_site: row.demo_site,
        benefit_note: row.benefit_note,
    }
}

fn empty_indicators() -> IndicatorView {
    IndicatorView {
        paper_international: 0,
        paper_domestic: 0,
        paper_provincial: 0,
        paper_core: 0,
        paper_general: 0,
        report_tech: 0,
        report_research: 0,
        report_transfer: 0,
        report_other: 0,
        monograph: 0,
        patent_invention_applied: 0,
        patent_invention_granted: 0,
        patent_utility_applied: 0,
        patent_utility_granted: 0,
        patent_design_applied: 0,
        patent_design_granted: 0,
        patent_pct_applied: 0,
        patent_pct_granted: 0,
        patent_other_applied: 0,
        patent_other_granted: 0,
        software_copyright_applied: 0,
        software_copyright_registered: 0,
        ic_layout_applied: 0,
        ic_layout_registered: 0,
        plant_variety_applied: 0,
        plant_variety_registered: 0,
        standard_national_lead: 0,
        standard_national_participate: 0,
        standard_industry_lead: 0,
        standard_industry_participate: 0,
        standard_local_lead: 0,
        standard_local_participate: 0,
        standard_enterprise_lead: 0,
        standard_enterprise_participate: 0,
        pilot_line: 0,
        production_line: 0,
        new_sales_income: "0.00".to_owned(),
        new_profit_tax: "0.00".to_owned(),
        talent_train_senior: 0,
        talent_train_intermediate: 0,
        talent_introduce_senior: 0,
        new_product: 0,
        new_technology: 0,
        new_process: 0,
        new_material: 0,
        support_platform: 0,
        test_base: 0,
        demo_site: 0,
        benefit_note: None,
    }
}

fn budget_sum(
    subjects: &[budget_subjects::Model],
    lines: &[application_budgets::Model],
    group: &str,
    only_fillable: bool,
) -> sea_orm::prelude::Decimal {
    lines.iter().fold(sea_orm::prelude::Decimal::ZERO, |sum, line| {
        let Some(subject) = subjects.iter().find(|item| item.code == line.subject_code) else {
            return sum;
        };
        let matched = subject.group_name == group && (!only_fillable || subject.fillable) && (only_fillable || subject.parent_code.is_none());
        if matched { sum + line.amount } else { sum }
    })
}

async fn touch(txn: &DatabaseTransaction, id: Uuid) -> Result<(), AppError> {
    applications::Entity::update_many()
        .col_expr(applications::Column::UpdatedAt, Expr::value(now()))
        .filter(applications::Column::Id.eq(id))
        .exec(txn)
        .await?;
    Ok(())
}

async fn load_owned(db: &DatabaseConnection, user: &CurrentUser, id: Uuid) -> Result<applications::Model, AppError> {
    let app = applications::Entity::find_by_id(id)
        .one(db)
        .await?
        .ok_or_else(|| AppError::NotFound("申报书不存在".to_owned()))?;
    if app.principal_id != user.id {
        return Err(AppError::Forbidden("只能查看本人负责的申报书".to_owned()));
    }
    Ok(app)
}

fn ensure_editable(app: &applications::Model) -> Result<(), AppError> {
    if matches!(
        app.status,
        ApplicationStatus::Draft | ApplicationStatus::DeptReturned | ApplicationStatus::OfficeReturned
    ) {
        Ok(())
    } else {
        Err(AppError::Conflict("当前状态不能修改申报书".to_owned()))
    }
}

fn ensure_section_class(app: &applications::Model, section: &str) -> Result<(), AppError> {
    let social = matches!(section, "ss-achievements" | "ss-design" | "ss-foundation" | "ss-outputs");
    let natural = matches!(
        section,
        "ns-profile" | "ns-content" | "ns-indicators" | "ns-schedules" | "ns-equipments"
    );
    if social && app.project_class != ProjectClass::Social {
        return Err(invalid("section", "这个栏目只用于社会科学类申报书"));
    }
    if natural && app.project_class == ProjectClass::Social {
        return Err(invalid("section", "这个栏目只用于自然科学类申报书"));
    }
    Ok(())
}

async fn open_cycle(db: &DatabaseConnection) -> Result<application_cycles::Model, AppError> {
    let timestamp = now();
    let cycles = application_cycles::Entity::find()
        .filter(application_cycles::Column::Status.eq(CycleStatus::Open))
        .order_by_desc(application_cycles::Column::CycleYear)
        .all(db)
        .await?;
    cycles
        .into_iter()
        .find(|cycle| {
            cycle.opens_at.is_none_or(|opens| opens <= timestamp) && cycle.closes_at.is_none_or(|closes| timestamp <= closes)
        })
        .ok_or_else(|| AppError::Conflict("当前没有开放中的申报批次".to_owned()))
}

fn parse_class(raw: &str) -> Result<ProjectClass, AppError> {
    match raw {
        "social" => Ok(ProjectClass::Social),
        "natural" => Ok(ProjectClass::Natural),
        "dual_high" => Ok(ProjectClass::DualHigh),
        _ => Err(invalid("project_class", "项目类别无效")),
    }
}

fn parse_family(raw: &str) -> Result<BudgetFamily, AppError> {
    match raw {
        "social" => Ok(BudgetFamily::Social),
        "natural" => Ok(BudgetFamily::Natural),
        _ => Err(invalid("family", "经费类别无效")),
    }
}

fn family_of(class: &ProjectClass) -> BudgetFamily {
    match class {
        ProjectClass::Social => BudgetFamily::Social,
        ProjectClass::Natural | ProjectClass::DualHigh => BudgetFamily::Natural,
    }
}

fn class_code(class: &ProjectClass) -> &'static str {
    match class {
        ProjectClass::Social => "social",
        ProjectClass::Natural => "natural",
        ProjectClass::DualHigh => "dual_high",
    }
}

fn status_code(status: &ApplicationStatus) -> &'static str {
    match status {
        ApplicationStatus::Draft => "draft",
        ApplicationStatus::Submitted => "submitted",
        ApplicationStatus::DeptReturned => "dept_returned",
        ApplicationStatus::DeptRejected => "dept_rejected",
        ApplicationStatus::DeptApproved => "dept_approved",
        ApplicationStatus::OfficeReturned => "office_returned",
        ApplicationStatus::OfficeRejected => "office_rejected",
        ApplicationStatus::OfficeApproved => "office_approved",
        ApplicationStatus::ExpertReviewing => "expert_reviewing",
        ApplicationStatus::ExpertRejected => "expert_rejected",
        ApplicationStatus::Established => "established",
    }
}

fn keyword_count(raw: &str) -> usize {
    raw.split([';', '；']).map(str::trim).filter(|item| !item.is_empty()).count()
}

fn needs_recommender(title: &str, education: &str, degree: &str) -> bool {
    !has_midlevel_title(title) && !has_graduate(education, degree)
}

fn has_midlevel_title(title: &str) -> bool {
    let title = title.trim();
    if title.is_empty() || title.contains("助教") || title.starts_with("助理") {
        return false;
    }
    ["讲师", "副教授", "教授", "研究员", "高级工程师", "高级实验师"]
        .iter()
        .any(|item| title.contains(item))
}

fn has_graduate(education: &str, degree: &str) -> bool {
    let text = format!("{education}{degree}");
    ["研究生", "硕士", "博士"].iter().any(|item| text.contains(item))
}

fn decode<T: for<'de> Deserialize<'de>>(body: Value) -> Result<T, AppError> {
    serde_json::from_value(body).map_err(|_| AppError::BadRequest("申报栏目数据格式不正确".to_owned()))
}

fn clean(value: Option<String>) -> Option<String> {
    value.and_then(|item| {
        let trimmed = item.trim();
        if trimmed.is_empty() { None } else { Some(trimmed.to_owned()) }
    })
}

fn limited(value: Option<String>, max: usize, field: &str, message: &str) -> Result<Option<String>, AppError> {
    let Some(value) = clean(value) else {
        return Ok(None);
    };
    if value.chars().count() > max {
        return Err(invalid(field, message));
    }
    Ok(Some(value))
}

fn require_len(value: &str, max: usize, field: &str, message: &str) -> Result<String, AppError> {
    let value = value.trim();
    if value.is_empty() {
        return Err(invalid(field, "这一项不能为空"));
    }
    if value.chars().count() > max {
        return Err(invalid(field, message));
    }
    Ok(value.to_owned())
}

const DATE_HINT: &str = "请写成 1976-12、1976-12-02，或纯数字 197612、19761202";

fn parse_date(raw: Option<&str>, field: &str) -> Result<Option<NaiveDate>, AppError> {
    loose_date(raw).map_err(|message| invalid(field, message))
}

fn take_row_date(raw: Option<&str>, index: usize, key: &str, label: &str) -> Result<Option<NaiveDate>, AppError> {
    loose_date(raw).map_err(|reason| invalid(&format!("schedules.{index}.{key}"), &format!("第 {} 条进度的{label}{reason}", index + 1)))
}

fn take_date(issues: &mut Vec<FieldError>, raw: Option<&str>, field: &str, label: &str) -> Option<NaiveDate> {
    match loose_date(raw) {
        Ok(date) => date,
        Err(reason) => {
            push_issue(issues, field, &format!("{label}{reason}"));
            None
        }
    }
}

fn take_text(issues: &mut Vec<FieldError>, value: Option<String>, max: usize, field: &str, message: &str) -> Option<String> {
    let value = clean(value)?;
    if value.chars().count() > max {
        push_issue(issues, field, message);
        return None;
    }
    Some(value)
}

fn take_required(issues: &mut Vec<FieldError>, value: &str, max: usize, field: &str, message: &str) -> Option<String> {
    let value = value.trim();
    if value.is_empty() {
        push_issue(issues, field, message);
        return None;
    }
    if value.chars().count() > max {
        push_issue(issues, field, message);
        return None;
    }
    Some(value.to_owned())
}

fn push_issue(issues: &mut Vec<FieldError>, field: &str, message: &str) {
    issues.push(FieldError {
        field: field.to_owned(),
        message: message.to_owned(),
    });
}

fn member_label(index: usize, name: &str) -> String {
    let name = name.trim();
    if name.is_empty() {
        format!("第 {} 位参加者", index + 1)
    } else {
        format!("第 {} 位参加者「{name}」", index + 1)
    }
}

fn loose_date(raw: Option<&str>) -> Result<Option<NaiveDate>, &'static str> {
    let Some(raw) = raw.map(str::trim).filter(|item| !item.is_empty()) else {
        return Ok(None);
    };
    let dashed = raw.replace(['/', '.'], "-");
    if let Ok(date) = NaiveDate::parse_from_str(&dashed, "%Y-%m-%d") {
        return Ok(Some(date));
    }
    if dashed.len() == 7 && dashed.as_bytes().get(4) == Some(&b'-') {
        if let Ok(date) = NaiveDate::parse_from_str(&format!("{dashed}-01"), "%Y-%m-%d") {
            return Ok(Some(date));
        }
    }
    if raw.chars().all(|item| item.is_ascii_digit()) && raw.len() == 8 {
        if let Ok(date) = NaiveDate::parse_from_str(raw, "%Y%m%d") {
            return Ok(Some(date));
        }
    }
    if raw.chars().all(|item| item.is_ascii_digit()) && raw.len() == 6 {
        if let Ok(date) = NaiveDate::parse_from_str(&format!("{raw}01"), "%Y%m%d") {
            return Ok(Some(date));
        }
    }
    Err(DATE_HINT)
}

fn read_money(raw: &str) -> Result<sea_orm::prelude::Decimal, &'static str> {
    let parsed = raw.trim().parse::<sea_orm::prelude::Decimal>().map_err(|_| "格式不正确，请填写数字，最多两位小数")?;
    if parsed < sea_orm::prelude::Decimal::ZERO {
        return Err("不能为负数");
    }
    if parsed.scale() > 2 {
        return Err("最多保留两位小数");
    }
    Ok(parsed)
}

fn money(raw: &str, field: &str) -> Result<sea_orm::prelude::Decimal, AppError> {
    read_money(raw).map_err(|message| invalid(field, message))
}

fn money_text(value: sea_orm::prelude::Decimal) -> String {
    format!("{value:.2}")
}

fn sort_of(index: usize) -> Result<i16, AppError> {
    i16::try_from(index + 1).map_err(|_| AppError::BadRequest("行数超出范围".to_owned()))
}

fn invalid(field: &str, message: &str) -> AppError {
    AppError::Validation(vec![FieldError {
        field: field.to_owned(),
        message: message.to_owned(),
    }])
}

async fn write_audit(state: &AppState, actor_id: Uuid, action: &'static str, id: Uuid) {
    let entry = AuditEntry {
        actor_id: Some(actor_id),
        action,
        target_type: "application",
        target_id: Some(id),
        detail: None,
        ip_address: None,
        user_agent: None,
    };
    if let Err(error) = record(&state.db, entry).await {
        tracing::error!(error = %error, "写入申报审计失败");
    }
}

#[cfg(test)]
mod tests {
    use super::{has_midlevel_title, keyword_count, loose_date, needs_recommender};

    #[test]
    fn keywords_split_on_both_semicolons() {
        assert_eq!(keyword_count("乡村文旅；高质量发展;评价体系"), 3);
        assert_eq!(keyword_count("  "), 0);
    }

    #[test]
    fn lecturer_with_graduate_education_skips_recommender() {
        assert!(!needs_recommender("讲师", "研究生", "硕士"));
    }

    #[test]
    fn assistant_without_degree_needs_recommender() {
        assert!(needs_recommender("助教", "本科", "学士"));
        assert!(!has_midlevel_title("助理研究员"));
    }

    #[test]
    fn birth_dates_accept_compact_digits() {
        assert_eq!(date_text("19761202"), "1976-12-02");
        assert_eq!(date_text("197809"), "1978-09-01");
        assert_eq!(date_text("1976-12"), "1976-12-01");
        assert_eq!(date_text("1976/12/02"), "1976-12-02");
        assert!(loose_date(Some("197613")).is_err());
        assert!(matches!(loose_date(Some("")), Ok(None)));
    }

    fn date_text(raw: &str) -> String {
        match loose_date(Some(raw)) {
            Ok(Some(date)) => date.to_string(),
            Ok(None) => panic!("日期不应为空"),
            Err(reason) => panic!("日期应能解析: {reason}"),
        }
    }
}

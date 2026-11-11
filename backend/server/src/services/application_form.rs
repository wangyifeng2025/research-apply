//! 按 2026 年院级申报书模板导出 PDF。
//!
//! 社会科学类使用附件 2 的栏目，自然科学类和双高专项使用附件 3。
//! 经费在系统里按万元保存，印刷稿的金额栏是元，导出时按 1 万元 = 10000 元换算。

use entity::application_cycles;
use entity::applications;
use entity::reviews;
use entity::sea_orm_active_enums::{ReviewDecision, ReviewStage, UserRole};
use sea_orm::prelude::Decimal;
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use uuid::Uuid;

use crate::auth::CurrentUser;
use crate::error::AppError;
use crate::services::applications::{
    ApplicationView, BudgetLineView, BudgetView, EquipmentView, IndicatorView, MemberView,
    OutputView, RecommenderView, load_view,
};
use crate::state::AppState;
use crate::support::audit::{AuditEntry, record};
use crate::support::cjk_pdf::{
    Cell, Document, PdfError, cell, head, head_span, mid, money_cell, shade_cell, span_cell,
    span_mid,
};

pub struct FormAccess {
    pub principal_id: Uuid,
    pub department_id: Option<Uuid>,
}

pub(crate) struct DepartmentNote {
    facts_accurate: Option<bool>,
    team_qualified: Option<bool>,
    resources_ready: Option<bool>,
    agree_to_manage: Option<bool>,
    decision: String,
    opinion: Option<String>,
    reviewed_on: Option<String>,
}

pub(crate) struct FormSource {
    pub view: ApplicationView,
    pub code: Option<String>,
    pub contact_phone: Option<String>,
    pub cycle_year: i32,
    pub commitment_on: Option<String>,
    pub department: Option<DepartmentNote>,
}

pub async fn export(
    state: &AppState,
    user: &CurrentUser,
    id: Uuid,
) -> Result<(String, Vec<u8>), AppError> {
    let app = applications::Entity::find_by_id(id)
        .one(&state.db)
        .await?
        .ok_or_else(|| AppError::NotFound("申报书不存在".to_owned()))?;
    may_export(
        user,
        &FormAccess {
            principal_id: app.principal_id,
            department_id: app.department_id,
        },
    )?;
    let source = load_source(&state.db, app).await?;
    let name = file_name(&source);
    let bytes = tokio::task::spawn_blocking(move || render(&source))
        .await
        .map_err(|_| AppError::Unavailable("导出申报书失败".to_owned()))?
        .map_err(|error| AppError::Unavailable(error.0))?;
    if let Err(error) = record(
        &state.db,
        AuditEntry {
            actor_id: Some(user.id),
            action: "application_exported",
            target_type: "application",
            target_id: Some(id),
            detail: None,
            ip_address: None,
            user_agent: None,
        },
    )
    .await
    {
        tracing::error!(error = %error, "写入申报书导出审计失败");
    }
    Ok((name, bytes))
}

pub fn may_export(user: &CurrentUser, access: &FormAccess) -> Result<(), AppError> {
    if user.has_role(UserRole::Admin) || user.has_role(UserRole::ResearchOffice) {
        return Ok(());
    }
    if user.id == access.principal_id {
        return Ok(());
    }
    if user.has_role(UserRole::DepartmentReviewer) {
        if user.department_id.is_some() && user.department_id == access.department_id {
            return Ok(());
        }
        return Err(AppError::Forbidden("只能导出本部门的申报书".to_owned()));
    }
    Err(AppError::Forbidden("无权导出这份申报书".to_owned()))
}

pub fn booklet_title(class: &str) -> &'static str {
    if class == "social" {
        "社会科学类项目申报书"
    } else {
        "自然科学类项目申报书"
    }
}

pub fn yuan_from_wan(text: &str) -> String {
    let wan: Decimal = text.parse().unwrap_or(Decimal::ZERO);
    format!("{:.2}", wan * Decimal::from(10_000))
}

pub(crate) fn file_name(source: &FormSource) -> String {
    let raw = source
        .view
        .title
        .as_deref()
        .filter(|item| !item.trim().is_empty())
        .unwrap_or("未命名");
    let safe: String = raw
        .chars()
        .map(|ch| match ch {
            '\\' | '/' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '_',
            control if control.is_control() => '_',
            other => other,
        })
        .take(40)
        .collect();
    format!("{}-{safe}.pdf", booklet_title(&source.view.project_class))
}

async fn load_source(
    db: &DatabaseConnection,
    app: applications::Model,
) -> Result<FormSource, AppError> {
    let id = app.id;
    let year = application_cycles::Entity::find_by_id(app.cycle_id)
        .one(db)
        .await?
        .map(|cycle| cycle.cycle_year)
        .unwrap_or(2026);
    let commitment_on = app
        .commitment_accepted_at
        .map(|value| value.format("%Y年%m月%d日").to_string());
    let contact_phone = app.contact_phone.clone().or(app.mobile_phone.clone());
    let code = app.code.clone();
    let department = department_note(db, id).await?;
    let view = load_view(db, id).await?;
    Ok(FormSource {
        view,
        code,
        contact_phone,
        cycle_year: year,
        commitment_on,
        department,
    })
}

async fn department_note(
    db: &DatabaseConnection,
    id: Uuid,
) -> Result<Option<DepartmentNote>, AppError> {
    let mut rows = reviews::Entity::find()
        .filter(reviews::Column::ApplicationId.eq(id))
        .filter(reviews::Column::Stage.eq(ReviewStage::Department))
        .all(db)
        .await?;
    rows.sort_by_key(|row| (row.reviewed_at, row.created_at));
    let Some(row) = rows.pop() else {
        return Ok(None);
    };
    Ok(Some(DepartmentNote {
        facts_accurate: row.facts_accurate,
        team_qualified: row.team_qualified,
        resources_ready: row.resources_ready,
        agree_to_manage: row.agree_to_manage,
        decision: decision_text(&row.decision).to_owned(),
        opinion: row.opinion,
        reviewed_on: row
            .reviewed_at
            .map(|value| value.format("%Y年%m月%d日").to_string()),
    }))
}

fn decision_text(decision: &ReviewDecision) -> &'static str {
    match decision {
        ReviewDecision::Approved => "通过",
        ReviewDecision::Returned => "退回修改",
        ReviewDecision::Rejected => "不通过",
        ReviewDecision::Pending => "待审核",
    }
}

pub(crate) fn render(source: &FormSource) -> Result<Vec<u8>, PdfError> {
    let social = source.view.project_class == "social";
    let mut doc = Document::new("黔西南民族职业技术学院院级科研项目申报书")?;
    write_cover(&mut doc, source);
    write_promise(&mut doc, source);
    write_instructions(&mut doc);
    doc.heading("一、基本情况");
    write_basic(&mut doc, source, !social);
    write_members(&mut doc, &source.view.members);
    if social {
        write_social(&mut doc, source);
    } else {
        write_profile(&mut doc, source.view.ns_profile.as_deref());
        write_natural(&mut doc, source);
    }
    doc.heading("六、经费预算");
    write_budget(&mut doc, source, !social);
    doc.heading("七、推荐人意见");
    write_recommender(&mut doc, source.view.recommender.as_ref());
    doc.heading("八、项目负责人所在部门审核意见");
    write_department(&mut doc, source.department.as_ref());
    doc.finish()
}

fn write_cover(doc: &mut Document, source: &FormSource) {
    let class = source.view.project_class.as_str();
    doc.left_line(
        if class == "social" {
            "附件2："
        } else {
            "附件3："
        },
        10.5,
    );
    doc.gap(6.0);
    doc.table(
        &[1.1, 1.5, 1.8, 1.0, 1.7],
        vec![vec![
            head("年  度"),
            mid(format!("{}年", source.cycle_year)),
            cell(""),
            head("编  号"),
            mid(source.code.clone().unwrap_or_default()),
        ]],
    );
    doc.gap(16.0);
    doc.center("黔西南民族职业技术学院科技计划", 16.0);
    doc.center(booklet_title(class), 18.0);
    doc.gap(8.0);
    doc.field("项  目  名  称：", &shown(&source.view.title));
    doc.field("项  目  类  别：", &category_line(class));
    doc.field("项 目 负 责 人：", &shown(&source.view.principal_name));
    doc.field("联  系  电  话：", &shown(&source.contact_phone));
    doc.field("负责人所在部门：", &shown(&source.view.department_name));
    doc.field("填  表  日  期：", &display_day(&source.view.filled_on));
    doc.gap(14.0);
    doc.center("科研处 制", 12.0);
    doc.center("2025年1月", 11.0);
}

fn write_promise(doc: &mut Document, source: &FormSource) {
    doc.heading("申请者承诺");
    doc.paragraph(
        "我承诺对本人填写的各项内容的真实性负责，保证没有知识产权争议。如获准立项，我承诺以本表为有约束力的协议，遵守黔西南民族职业技术学院科研项目管理的相关规定，按计划认真开展研究工作，取得预期研究成果。黔西南民族职业技术学院有权使用本表所有数据和资料。",
        10.5,
    );
    doc.gap(6.0);
    doc.paragraph(
        "作为项目研究者，本人完全了解黔西南民族职业技术学院科研项目管理的有关规定，完全意识到本声明的法律后果由本人承担。特授权黔西南民族职业技术学院：有权保留并向有关部门或机构报送项目成果的原件、复印件、摘要和电子版；有权公布项目研究成果的全部或部分内容，同意以影印、缩印、扫描、出版等形式复制、保存、汇编项目研究成果；允许项目研究成果被他人查阅和借阅；有权推广科研成果，允许将项目研究成果通过内部报告、学术会议、专业报刊、大众媒体、专门网站、评奖等形式进行宣传、试验和培训。",
        10.5,
    );
    doc.gap(10.0);
    doc.paragraph("申请人（签章）：", 11.0);
    doc.gap(16.0);
    if let Some(day) = &source.commitment_on {
        doc.paragraph(day, 11.0);
        doc.note("系统记录：申请人已在线确认以上承诺。提交纸质申报书时，仍需本人在此处签字。");
    } else {
        doc.paragraph("年    月    日", 11.0);
        doc.note("申请人尚未在系统中确认承诺。");
    }
}

fn write_instructions(doc: &mut Document) {
    doc.heading("填  报  说  明");
    doc.paragraph(
        "一、本表封面上方的“编号”栏申请人不填，其他栏目由申请人用中文如实填写。",
        10.0,
    );
    doc.paragraph("二、部分栏目填写说明：", 10.0);
    doc.paragraph(
        "项目名称：应准确、简明反映研究内容，最多不超过40个汉字。",
        10.0,
    );
    doc.paragraph("关 键 词：最多不超过5个，各词之间用分号隔开。", 10.0);
    doc.paragraph("工作单位：按单位公章填写全称。", 10.0);
    doc.paragraph(
        "主要参加者：必须真正参加本项目的研究工作，不含项目负责人，不包括科研管理、财务管理、后勤服务等人员，最多不超过5人。",
        10.0,
    );
    doc.paragraph("三、本表报送一式3份，统一采用A4纸印制，左侧装订。", 10.0);
    doc.paragraph("四、联系方式：", 10.0);
    doc.paragraph(
        "地　　址：贵州省兴义市丰都街道民航大道66号黔西南民族职业技术学院行政楼316室科研处",
        10.0,
    );
    doc.paragraph("邮政编码：562400", 10.0);
    doc.paragraph("电子邮箱：qxnzyky@163.com", 10.0);
    doc.paragraph("办公电话：0859-3655091", 10.0);
}

fn write_basic(doc: &mut Document, source: &FormSource, natural: bool) {
    let view = &source.view;
    let mut rows = Vec::new();
    if natural {
        rows.push(vec![head("项目名称"), span_cell(shown(&view.title), 7)]);
    }
    rows.push(vec![head("关 键 词"), span_cell(shown(&view.keywords), 7)]);
    rows.push(vec![
        head("项目类别"),
        span_cell(category_line(&view.project_class), 7),
    ]);
    rows.push(vec![
        head("项目负责人"),
        cell(shown(&view.principal_name)),
        head("性别"),
        cell(shown(&view.gender)),
        head("民族"),
        cell(shown(&view.ethnicity)),
        head("出生年月"),
        cell(shown(&view.birth_date)),
    ]);
    rows.push(vec![
        head("行政职务"),
        cell(shown(&view.admin_title)),
        head("专业职务"),
        span_cell(shown(&view.professional_title), 2),
        head("研究专长"),
        span_cell(shown(&view.research_expertise), 2),
    ]);
    rows.push(vec![
        head("最后学历"),
        cell(shown(&view.education)),
        head("最后学位"),
        span_cell(shown(&view.degree), 2),
        head("手机号码"),
        span_cell(shown(&view.mobile_phone), 2),
    ]);
    doc.table(&[1.2, 1.15, 0.8, 0.95, 0.8, 1.15, 1.05, 1.15], rows);
}

fn write_members(doc: &mut Document, members: &[MemberView]) {
    doc.subhead("主要参加者（限5人，不含项目负责人）");
    let mut rows = vec![vec![
        head("姓名"),
        head("出生年月"),
        head("专业职务"),
        head("研究专长"),
        head("学历"),
        head("学位"),
        head("工作单位/部门"),
    ]];
    let count = members.len().max(5);
    for index in 0..count {
        let member = members.get(index);
        rows.push(vec![
            cell(member.map(|item| item.name.clone()).unwrap_or_default()),
            cell(
                member
                    .map(|item| shown(&item.birth_date))
                    .unwrap_or_default(),
            ),
            cell(
                member
                    .map(|item| shown(&item.professional_title))
                    .unwrap_or_default(),
            ),
            cell(
                member
                    .map(|item| shown(&item.research_expertise))
                    .unwrap_or_default(),
            ),
            cell(
                member
                    .map(|item| shown(&item.education))
                    .unwrap_or_default(),
            ),
            cell(member.map(|item| shown(&item.degree)).unwrap_or_default()),
            cell(
                member
                    .map(|item| shown(&item.organization))
                    .unwrap_or_default(),
            ),
        ]);
    }
    doc.table(&[1.15, 1.15, 1.25, 1.45, 0.9, 0.9, 2.0], rows);
}

fn write_social(doc: &mut Document, source: &FormSource) {
    doc.heading("二、负责人和项目组成员近五年来已开展的相关工作和取得的研究成果");
    let mut rows = vec![vec![
        head("成果名称"),
        head("研究者"),
        head("成果形式"),
        head("发表或交流情况"),
        head("发表出版时间"),
    ]];
    let count = source.view.ss_achievements.len().max(5);
    for index in 0..count {
        let item = source.view.ss_achievements.get(index);
        rows.push(vec![
            cell(item.map(|row| row.title.clone()).unwrap_or_default()),
            cell(item.map(|row| shown(&row.researchers)).unwrap_or_default()),
            cell(item.map(|row| shown(&row.outcome_form)).unwrap_or_default()),
            cell(item.map(|row| shown(&row.publication)).unwrap_or_default()),
            cell(item.map(|row| shown(&row.published_on)).unwrap_or_default()),
        ]);
    }
    doc.table(&[2.4, 1.2, 1.2, 2.6, 1.4], rows);

    doc.heading("三、项目设计论证");
    write_labeled_boxes(
        doc,
        &[
            (
                "1.选题依据",
                "国内外相关研究学术梳理及研究动态，本项目相对已有研究的独到学术价值和应用价值等。",
                source.view.ss_design.topic_basis.as_deref(),
            ),
            (
                "2.研究内容",
                "本项目的研究假设、核心概念、研究对象、总体框架、重点难点、主要目标等。",
                source.view.ss_design.research_content.as_deref(),
            ),
            (
                "3.思路方法",
                "本项目研究的具体研究方法、研究计划及其可行性等。",
                source.view.ss_design.methodology.as_deref(),
            ),
            (
                "4.创新之处",
                "在学术思想、学术观点、研究方法、破解难题等方面的特色和创新。",
                source.view.ss_design.innovation.as_deref(),
            ),
            (
                "5.预期成果",
                "成果形式、使用去向及预期社会效益等。",
                source.view.ss_design.expected_outcome.as_deref(),
            ),
            (
                "6.参考文献",
                "开展本项目研究的主要中外参考文献。",
                source.view.ss_design.reference_list.as_deref(),
            ),
        ],
    );

    doc.heading("四、研究基础和条件保障");
    doc.note("本表参照以下提纲撰写，要求填写内容真实准确。");
    write_labeled_boxes(
        doc,
        &[
            (
                "1.学术简历",
                "项目负责人的主要学术经历，研究团队在选题相关研究领域的学术积累和贡献等。",
                source.view.ss_foundation.academic_resume.as_deref(),
            ),
            (
                "2.研究基础",
                "项目负责人前期相关代表性研究成果、核心观点及社会评价等。与本项目无关的成果不能作为前期成果填写。",
                source.view.ss_foundation.research_basis.as_deref(),
            ),
            (
                "3.承担项目",
                "负责人承担的各级各类科研项目情况，包括项目名称、资助金额、结项情况、研究起止时间等。",
                source.view.ss_foundation.undertaken_projects.as_deref(),
            ),
            (
                "4.条件保障",
                "完成本项目研究的时间保证、资料设备等科研条件。",
                source.view.ss_foundation.support_conditions.as_deref(),
            ),
        ],
    );

    doc.heading("五、预期研究成果");
    doc.subhead("主要阶段性成果（限报10项）");
    doc.table(
        &[0.7, 1.3, 3.0, 1.4, 1.3],
        output_rows("成果名称", &source.view.ss_stage_outputs, 10),
    );
    doc.gap(6.0);
    doc.subhead("最终研究成果（限报2项）");
    doc.table(
        &[0.7, 1.3, 3.0, 1.4, 1.3],
        output_rows("最终成果名称", &source.view.ss_final_outputs, 2),
    );
}

fn write_natural(doc: &mut Document, source: &FormSource) {
    doc.heading("二、主要研究内容");
    doc.note("目标与任务需求分析，主要技术难点和问题分析，研究与开发任务与内容，关键技术、技术难点、创新点等。");
    doc.boxed(source.view.ns_content.as_deref().unwrap_or(""), 96.0);

    doc.heading("三、主要技术经济指标");
    write_indicators(doc, &source.view.ns_indicators);

    doc.heading("四、计划进度及目标");
    let mut rows = vec![vec![head("开始日期"), head("结束日期"), head("阶段目标")]];
    let count = source.view.ns_schedules.len().max(5);
    for index in 0..count {
        let item = source.view.ns_schedules.get(index);
        rows.push(vec![
            cell(item.map(|row| shown(&row.start_date)).unwrap_or_default()),
            cell(item.map(|row| shown(&row.end_date)).unwrap_or_default()),
            cell(item.map(|row| shown(&row.goal)).unwrap_or_default()),
        ]);
    }
    doc.table(&[1.3, 1.3, 3.4], rows);

    doc.heading("五、仪器设备购买清单（5000元以上/台·套）");
    write_equipment(doc, &source.view.ns_equipments);
}

fn write_profile(doc: &mut Document, summary: Option<&str>) {
    doc.subhead("项目简介（主要研究内容，主要技术经济等指标）");
    doc.boxed(summary.unwrap_or(""), 72.0);
}

fn write_indicators(doc: &mut Document, item: &IndicatorView) {
    doc.subhead("（一）量化考核指标");
    doc.subhead("1.科技产出");
    doc.note("论文按填报页对应关系导出：核心期刊只计国际、国内，省级不设核心期刊。");
    doc.table(
        &[1.3, 1.2, 1.0, 1.0, 1.0, 1.0],
        vec![
            vec![
                head("论文（篇）"),
                head("期刊"),
                head("国际"),
                head("国内"),
                head_span("省级", 2),
            ],
            vec![
                cell(""),
                head("核心期刊"),
                mid(count(item.paper_core)),
                mid(count(item.paper_domestic)),
                shade_cell("—", 2),
            ],
            vec![
                cell(""),
                head("一般期刊"),
                mid(count(item.paper_general)),
                mid(count(item.paper_international)),
                span_mid(count(item.paper_provincial), 2),
            ],
            vec![
                head("报告（篇）"),
                head("科技报告"),
                head("研究报告"),
                head_span("科技成果转化", 2),
                head("其他报告"),
            ],
            vec![
                cell(""),
                mid(count(item.report_tech)),
                mid(count(item.report_research)),
                span_mid(count(item.report_transfer), 2),
                mid(count(item.report_other)),
            ],
            vec![head("专著（部）"), span_cell(count(item.monograph), 5)],
        ],
    );
    doc.gap(4.0);
    doc.table(
        &[1.35, 1.25, 1.0, 1.15, 1.15, 0.9],
        vec![
            vec![
                head("知识产权（件）"),
                head("专利"),
                head("发明"),
                head("实用新型"),
                head("外观设计"),
                head("PCT"),
            ],
            vec![
                cell(""),
                head("申请（受理）数"),
                mid(count(item.patent_invention_applied)),
                mid(count(item.patent_utility_applied)),
                mid(count(item.patent_design_applied)),
                mid(count(item.patent_pct_applied)),
            ],
            vec![
                cell(""),
                head("授权数"),
                mid(count(item.patent_invention_granted)),
                mid(count(item.patent_utility_granted)),
                mid(count(item.patent_design_granted)),
                mid(count(item.patent_pct_granted)),
            ],
            vec![
                cell(""),
                head("其他"),
                head("软件著作权"),
                head("集成电路布图"),
                head_span("植物新品种", 2),
            ],
            vec![
                cell(""),
                head("申请数"),
                mid(count(item.software_copyright_applied)),
                mid(count(item.ic_layout_applied)),
                span_mid(count(item.plant_variety_applied), 2),
            ],
            vec![
                cell(""),
                head("登记数"),
                mid(count(item.software_copyright_registered)),
                mid(count(item.ic_layout_registered)),
                span_mid(count(item.plant_variety_registered), 2),
            ],
        ],
    );
    doc.gap(4.0);
    doc.table(
        &[1.2, 1.15, 1.05, 1.05, 1.05, 1.05],
        vec![
            vec![
                head("标准（项）"),
                head("标准分类"),
                head("国家标准"),
                head("行业标准"),
                head("地方标准"),
                head("企业标准"),
            ],
            vec![
                cell(""),
                head("牵头"),
                mid(count(item.standard_national_lead)),
                mid(count(item.standard_industry_lead)),
                mid(count(item.standard_local_lead)),
                mid(count(item.standard_enterprise_lead)),
            ],
            vec![
                cell(""),
                head("参与"),
                mid(count(item.standard_national_participate)),
                mid(count(item.standard_industry_participate)),
                mid(count(item.standard_local_participate)),
                mid(count(item.standard_enterprise_participate)),
            ],
            vec![head("中试数（条）"), span_cell(count(item.pilot_line), 5)],
            vec![
                head("生产线（条）"),
                span_cell(count(item.production_line), 5),
            ],
            vec![
                head("累计新增销售收入（万元）"),
                span_cell(item.new_sales_income.clone(), 5),
            ],
            vec![
                head("累计新增利税（万元）"),
                span_cell(item.new_profit_tax.clone(), 5),
            ],
        ],
    );
    doc.subhead("2.研发服务能力");
    doc.table(
        &[1.55, 1.25, 1.25, 1.35, 1.15],
        vec![
            vec![
                head("培养、引进人才（人）"),
                head("培养高级职称以上"),
                head("培养中级职称以上"),
                head_span("引进高级职称以上", 2),
            ],
            vec![
                cell(""),
                mid(count(item.talent_train_senior)),
                mid(count(item.talent_train_intermediate)),
                span_mid(count(item.talent_introduce_senior), 2),
            ],
            vec![
                head("引进新产品、新技术、新材料"),
                head("新产品（种）"),
                head("新技术（项）"),
                head("新工艺（项）"),
                head("新材料（项）"),
            ],
            vec![
                cell(""),
                mid(count(item.new_product)),
                mid(count(item.new_technology)),
                mid(count(item.new_process)),
                mid(count(item.new_material)),
            ],
            vec![
                head("其他指标"),
                head("支撑平台（个）"),
                head("试验基地（个）"),
                head_span("示范点（区）（个）", 2),
            ],
            vec![
                cell(""),
                mid(count(item.support_platform)),
                mid(count(item.test_base)),
                span_mid(count(item.demo_site), 2),
            ],
        ],
    );
    doc.subhead("（二）主要技术经济指标及社会效益说明");
    doc.boxed(item.benefit_note.as_deref().unwrap_or(""), 72.0);
}

fn write_equipment(doc: &mut Document, items: &[EquipmentView]) {
    let mut rows = vec![vec![
        head("序号"),
        head("仪器名称及规格型号"),
        head("数量"),
        head("单价"),
        head("金额（元）"),
        head("资金来源"),
        head("用途"),
    ]];
    let mut total = Decimal::ZERO;
    let slots = items.len().max(5);
    for index in 0..slots {
        let Some(item) = items.get(index) else {
            rows.push(vec![
                mid(format!("{}", index + 1)),
                cell(""),
                cell(""),
                cell(""),
                cell(""),
                cell(""),
                cell(""),
            ]);
            continue;
        };
        total += item.amount.parse::<Decimal>().unwrap_or(Decimal::ZERO);
        rows.push(vec![
            mid(format!("{}", index + 1)),
            cell(item.name_spec.clone()),
            mid(count(item.quantity)),
            money_cell(item.unit_price.clone()),
            money_cell(item.amount.clone()),
            cell(shown(&item.funding_source)),
            cell(shown(&item.purpose)),
        ]);
    }
    rows.push(vec![
        span_cell("合计（元）", 4),
        money_cell(format!("{total:.2}")),
        span_cell("", 2),
    ]);
    doc.table(&[0.7, 2.3, 0.7, 1.0, 1.1, 1.1, 1.3], rows);
}

fn write_budget(doc: &mut Document, source: &FormSource, numbered: bool) {
    let mut rows = Vec::new();
    if numbered {
        rows.push(vec![head("序号"), head("预算科目名称"), head("金额（元）")]);
    } else {
        rows.push(vec![head("支出科目"), head("金额（元）")]);
    }
    for (index, line) in source.view.budget.lines.iter().enumerate() {
        let amount = budget_yuan(line, &source.view.budget);
        if numbered {
            rows.push(vec![
                mid(format!("{}", index + 1)),
                cell(line.name.clone()),
                money_cell(amount),
            ]);
        } else {
            rows.push(vec![cell(line.name.clone()), money_cell(amount)]);
        }
    }
    if numbered {
        doc.table(&[0.8, 4.2, 1.5], rows);
    } else {
        doc.table(&[4.4, 1.6], rows);
    }
    doc.gap(4.0);
    doc.note("金额单位为元。系统按万元保存，导出时按 1 万元 = 10000 元换算。绩效支出含在间接费用中，不重复计入经费合计。");
}

fn write_recommender(doc: &mut Document, person: Option<&RecommenderView>) {
    doc.paragraph(
        "对不具备中级及以上职称或研究生及以上学历的申请人，须由所从事研究领域具有副高级以上专业技术职称的同行专家推荐。推荐人须认真负责地介绍项目负责人和参加者的专业水平、科研能力、科研态度和科研条件，并说明该项目取得预期成果的可能性。推荐人不能为项目负责人及参加者。",
        10.0,
    );
    doc.gap(4.0);
    doc.field(
        "推荐人姓名：",
        person.map(|item| item.name.as_str()).unwrap_or(""),
    );
    doc.field(
        "专业技术职务：",
        person
            .map(|item| item.professional_title.as_str())
            .unwrap_or(""),
    );
    doc.field(
        "研究专长：",
        person.map(|item| item.expertise.as_str()).unwrap_or(""),
    );
    doc.field(
        "工作单位：",
        person.map(|item| item.organization.as_str()).unwrap_or(""),
    );
    doc.subhead("推荐意见");
    doc.boxed(
        person
            .and_then(|item| item.opinion.as_deref())
            .unwrap_or(""),
        72.0,
    );
    doc.gap(4.0);
    let signed = person
        .and_then(|item| item.signed_on.as_deref())
        .map(display_raw_day)
        .unwrap_or_else(|| "年    月    日".to_owned());
    doc.paragraph(&signed, 11.0);
}

fn write_department(doc: &mut Document, note: Option<&DepartmentNote>) {
    doc.paragraph(
        "申报书所填写的内容是否属实；该项目负责人及参加者的政治和业务素质是否适合承担本项目的研究工作；本部门能否提供完成本项目所需的时间和条件；本部门是否同意承担本项目的管理任务和信誉保证。",
        10.0,
    );
    doc.gap(4.0);
    if let Some(note) = note {
        let body = format!(
            "申报内容是否属实：{}\n负责人及参加者是否适合承担本项目：{}\n本部门能否提供完成项目所需的时间和条件：{}\n本部门是否同意承担管理任务和信誉保证：{}\n审核结论：{}\n审核意见：{}",
            yes_no(note.facts_accurate),
            yes_no(note.team_qualified),
            yes_no(note.resources_ready),
            yes_no(note.agree_to_manage),
            note.decision,
            note.opinion.as_deref().unwrap_or("（未填写）")
        );
        doc.boxed(&body, 88.0);
        doc.gap(8.0);
        doc.paragraph("部门公章", 11.0);
        doc.gap(10.0);
        doc.paragraph(
            note.reviewed_on.as_deref().unwrap_or("年    月    日"),
            11.0,
        );
    } else {
        doc.boxed("部门公章\n\n年    月    日", 96.0);
        doc.note("系审核完成后，导出文件会写入四项确认、结论和意见。");
    }
}

fn write_labeled_boxes(doc: &mut Document, blocks: &[(&str, &str, Option<&str>)]) {
    for (title, prompt, body) in blocks {
        doc.subhead(title);
        doc.note(prompt);
        doc.boxed(body.unwrap_or(""), 64.0);
        doc.gap(3.0);
    }
}

fn output_rows(title: &str, items: &[OutputView], minimum: usize) -> Vec<Vec<Cell>> {
    let mut rows = vec![vec![
        head("序号"),
        head("完成时间"),
        head(title),
        head("成果形式"),
        head("负责人"),
    ]];
    let count = items.len().max(minimum);
    for index in 0..count {
        let item = items.get(index);
        rows.push(vec![
            mid(format!("{}", index + 1)),
            cell(item.map(|row| shown(&row.complete_on)).unwrap_or_default()),
            cell(item.map(|row| row.title.clone()).unwrap_or_default()),
            cell(item.map(|row| shown(&row.outcome_form)).unwrap_or_default()),
            cell(item.map(|row| shown(&row.owner_name)).unwrap_or_default()),
        ]);
    }
    rows
}

fn budget_yuan(line: &BudgetLineView, budget: &BudgetView) -> String {
    let wan = if line.group_name == "total" {
        budget.total.clone()
    } else if line.parent_code.is_none() && line.group_name == "direct" {
        budget.direct_total.clone()
    } else if line.parent_code.is_none() && line.group_name == "indirect" {
        budget.indirect_total.clone()
    } else if !line.fillable {
        let sum = budget
            .lines
            .iter()
            .filter(|item| item.parent_code.as_deref() == Some(line.code.as_str()))
            .fold(Decimal::ZERO, |sum, item| {
                sum + item.amount.parse::<Decimal>().unwrap_or(Decimal::ZERO)
            });
        format!("{sum:.2}")
    } else {
        line.amount.clone()
    };
    let yuan = yuan_from_wan(&wan);
    if line.fillable && yuan == "0.00" {
        String::new()
    } else {
        yuan
    }
}

fn category_line(class: &str) -> String {
    match class {
        "social" => "B.社会科学类".to_owned(),
        "dual_high" => "A.自然科学类    【已选】D.“双高”校建设专项".to_owned(),
        _ => "【已选】A.自然科学类    D.“双高”校建设专项".to_owned(),
    }
}

fn yes_no(value: Option<bool>) -> &'static str {
    match value {
        Some(true) => "是",
        Some(false) => "否",
        None => "未填写",
    }
}

fn count(value: i32) -> String {
    value.to_string()
}

fn shown(value: &Option<String>) -> String {
    value.clone().unwrap_or_default()
}

fn display_day(value: &Option<String>) -> String {
    value.as_deref().map(display_raw_day).unwrap_or_default()
}

fn display_raw_day(value: &str) -> String {
    chrono::NaiveDate::parse_from_str(value, "%Y-%m-%d")
        .map(|date| date.format("%Y年%m月%d日").to_string())
        .unwrap_or_else(|_| value.to_owned())
}

#[cfg(test)]
mod tests {
    use super::{
        DepartmentNote, FormAccess, FormSource, booklet_title, file_name, may_export, render,
        yuan_from_wan,
    };
    use crate::auth::CurrentUser;
    use crate::services::applications::{
        AchievementView, ApplicationView, BudgetLineView, BudgetView, DesignView, EquipmentView,
        FoundationView, MemberView, OutputView, ScheduleView, empty_indicators,
    };
    use entity::sea_orm_active_enums::UserRole;
    use uuid::Uuid;

    #[test]
    fn wan_amounts_convert_to_yuan() {
        assert_eq!(yuan_from_wan("1.25"), "12500.00");
        assert_eq!(yuan_from_wan("0.10"), "1000.00");
        assert_eq!(yuan_from_wan("0"), "0.00");
        assert_eq!(yuan_from_wan("不是数字"), "0.00");
    }

    #[test]
    fn booklet_follows_the_printed_template() {
        assert_eq!(booklet_title("social"), "社会科学类项目申报书");
        assert_eq!(booklet_title("natural"), "自然科学类项目申报书");
        assert_eq!(booklet_title("dual_high"), "自然科学类项目申报书");
    }

    #[test]
    fn export_permission_hides_the_form_from_other_people() {
        let owner = Uuid::new_v4();
        let department = Uuid::new_v4();
        let access = FormAccess {
            principal_id: owner,
            department_id: Some(department),
        };
        assert!(
            may_export(
                &person(owner, vec![UserRole::Teacher], Some(department)),
                &access
            )
            .is_ok()
        );
        assert!(
            may_export(
                &person(Uuid::new_v4(), vec![UserRole::ResearchOffice], None),
                &access
            )
            .is_ok()
        );
        assert!(
            may_export(
                &person(Uuid::new_v4(), vec![UserRole::Admin], None),
                &access
            )
            .is_ok()
        );
        assert!(
            may_export(
                &person(
                    Uuid::new_v4(),
                    vec![UserRole::DepartmentReviewer],
                    Some(department)
                ),
                &access,
            )
            .is_ok()
        );
        assert!(
            may_export(
                &person(
                    Uuid::new_v4(),
                    vec![UserRole::DepartmentReviewer],
                    Some(Uuid::new_v4())
                ),
                &access,
            )
            .is_err()
        );
        assert!(
            may_export(
                &person(Uuid::new_v4(), vec![UserRole::Expert], None),
                &access
            )
            .is_err()
        );
        assert!(
            may_export(
                &person(Uuid::new_v4(), vec![UserRole::Teacher], None),
                &access
            )
            .is_err()
        );
    }

    #[test]
    fn rendered_forms_are_pdfs_with_template_titles() {
        let social = render(&sample("social")).expect("社科申报书应能生成");
        let natural = render(&sample("natural")).expect("自科申报书应能生成");
        assert!(social.starts_with(b"%PDF"));
        assert!(natural.starts_with(b"%PDF"));
        assert!(social.len() > 2_000);
        assert!(natural.len() > 2_000);
        assert!(has_unicode(&social, '社'));
        assert!(has_unicode(&social, '图'));
        assert!(has_unicode(&natural, '自'));
        assert!(has_unicode(&natural, '设'));
    }

    fn has_unicode(pdf: &[u8], ch: char) -> bool {
        let needle = format!("<{:04X}>", ch as u32);
        pdf.windows(needle.len())
            .any(|window| window == needle.as_bytes())
    }

    fn person(id: Uuid, roles: Vec<UserRole>, department_id: Option<Uuid>) -> CurrentUser {
        CurrentUser {
            id,
            username: "tester".to_owned(),
            real_name: "测试".to_owned(),
            roles,
            department_id,
        }
    }

    fn sample(class: &str) -> FormSource {
        let mut indicators = empty_indicators();
        indicators.paper_core = 1;
        indicators.paper_domestic = 2;
        indicators.monograph = 1;
        indicators.benefit_note =
            Some("形成一套可在县域推广的技术规程，并培训一线教师。".to_owned());
        FormSource {
            view: ApplicationView {
                id: Uuid::nil(),
                project_class: class.to_owned(),
                title: Some("县域产业协同创新研究".to_owned()),
                keywords: Some("县域；产业；协同".to_owned()),
                status: "draft".to_owned(),
                editable: true,
                principal_name: Some("赵明远".to_owned()),
                gender: Some("男".to_owned()),
                ethnicity: Some("汉族".to_owned()),
                birth_date: Some("1985-06".to_owned()),
                admin_title: Some("教研室主任".to_owned()),
                professional_title: Some("副教授".to_owned()),
                research_expertise: Some("区域经济".to_owned()),
                education: Some("研究生".to_owned()),
                degree: Some("硕士".to_owned()),
                mobile_phone: Some("13700001001".to_owned()),
                department_name: Some("经济管理学院".to_owned()),
                filled_on: Some("2026-03-18".to_owned()),
                commitment_accepted: true,
                members: vec![MemberView {
                    name: "陈霞".to_owned(),
                    birth_date: Some("1990-04".to_owned()),
                    professional_title: Some("讲师".to_owned()),
                    research_expertise: Some("计算机应用".to_owned()),
                    education: Some("研究生".to_owned()),
                    degree: Some("硕士".to_owned()),
                    organization: Some("信息工程学院".to_owned()),
                }],
                recommender: None,
                budget: BudgetView {
                    lines: vec![
                        BudgetLineView {
                            code: "ss_direct".to_owned(),
                            name: "一、直接费用".to_owned(),
                            group_name: "direct".to_owned(),
                            parent_code: None,
                            fillable: false,
                            amount: "0.00".to_owned(),
                        },
                        BudgetLineView {
                            code: "ss_books".to_owned(),
                            name: "1.图书资料费".to_owned(),
                            group_name: "direct".to_owned(),
                            parent_code: Some("ss_direct".to_owned()),
                            fillable: true,
                            amount: "1.25".to_owned(),
                        },
                        BudgetLineView {
                            code: "ss_indirect".to_owned(),
                            name: "二、间接费用".to_owned(),
                            group_name: "indirect".to_owned(),
                            parent_code: None,
                            fillable: true,
                            amount: "0.30".to_owned(),
                        },
                        BudgetLineView {
                            code: "ss_total".to_owned(),
                            name: "经费合计".to_owned(),
                            group_name: "total".to_owned(),
                            parent_code: None,
                            fillable: false,
                            amount: "0.00".to_owned(),
                        },
                    ],
                    direct_total: "1.25".to_owned(),
                    indirect_total: "0.30".to_owned(),
                    total: "1.55".to_owned(),
                },
                ss_achievements: vec![AchievementView {
                    title: "县域旅游协同发展研究".to_owned(),
                    researchers: Some("赵明远".to_owned()),
                    outcome_form: Some("论文".to_owned()),
                    publication: Some("贵州社会科学".to_owned()),
                    published_on: Some("2024-06".to_owned()),
                }],
                ss_design: DesignView {
                    topic_basis: Some(
                        "围绕县域产业升级，梳理近五年相关研究，说明本项目的应用价值。".to_owned(),
                    ),
                    research_content: Some(
                        "提出协同创新的分析框架，并完成两个县域的比较研究。".to_owned(),
                    ),
                    ..DesignView::default()
                },
                ss_foundation: FoundationView::default(),
                ss_stage_outputs: vec![OutputView {
                    complete_on: Some("2026-12".to_owned()),
                    title: "阶段调研报告".to_owned(),
                    outcome_form: Some("报告".to_owned()),
                    owner_name: Some("赵明远".to_owned()),
                }],
                ss_final_outputs: Vec::new(),
                ns_profile: Some(
                    "研究县域产业数据采集与分析平台，形成可复制的技术方案。".to_owned(),
                ),
                ns_content: Some(
                    "分析数据采集、清洗和可视化中的关键技术，完成原型系统。".repeat(8),
                ),
                ns_indicators: indicators,
                ns_schedules: vec![ScheduleView {
                    start_date: Some("2026-04-01".to_owned()),
                    end_date: Some("2026-08-31".to_owned()),
                    goal: Some("完成需求调研和总体设计".to_owned()),
                }],
                ns_equipments: vec![EquipmentView {
                    name_spec: "图形工作站 Dell Precision".to_owned(),
                    quantity: 1,
                    unit_price: "12000.00".to_owned(),
                    amount: "12000.00".to_owned(),
                    funding_source: Some("项目经费".to_owned()),
                    purpose: Some("模型训练".to_owned()),
                }],
                conclusion_opinion: None,
                attachments: Vec::new(),
            },
            code: Some("2026-B-001".to_owned()),
            contact_phone: Some("13700001001".to_owned()),
            cycle_year: 2026,
            commitment_on: Some("2026年03月18日".to_owned()),
            department: Some(DepartmentNote {
                facts_accurate: Some(true),
                team_qualified: Some(true),
                resources_ready: Some(true),
                agree_to_manage: Some(true),
                decision: "通过".to_owned(),
                opinion: Some("同意推荐申报。".to_owned()),
                reviewed_on: Some("2026年04月02日".to_owned()),
            }),
        }
    }

    #[test]
    fn file_name_keeps_the_project_title() {
        let source = sample("social");
        assert_eq!(
            file_name(&source),
            "社会科学类项目申报书-县域产业协同创新研究.pdf"
        );
    }
}

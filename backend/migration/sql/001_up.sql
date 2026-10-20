-- @@
-- 院级科研申报：账号、批次、申报主表，社会科学 / 自然科学分表，审核、通知与审计。
-- 不使用触发器和存储函数：数据库只保留声明式约束，业务规则由服务层校验。
-- 用户角色：教师、系审核人、科研处、专家、管理员
CREATE TYPE user_role AS ENUM (
    'teacher',
    'department_reviewer',
    'research_office',
    'expert',
    'admin'
);

-- @@
-- 项目类别：社会科学、自然科学、双高项目
CREATE TYPE project_class AS ENUM ('social', 'natural', 'dual_high');

-- @@
-- 批次状态：草稿、开放、关闭
CREATE TYPE cycle_status AS ENUM ('draft', 'open', 'closed');

-- @@
-- 申报状态：草稿、提交、系审核退回 / 不通过 / 通过、科研处退回 / 不通过 / 通过、专家盲审中、不予立项、立项
CREATE TYPE application_status AS ENUM (
    'draft',
    'submitted',
    'dept_returned',
    'dept_rejected',
    'dept_approved',
    'office_returned',
    'office_rejected',
    'office_approved',
    'expert_reviewing',
    'expert_rejected',
    'established'
);

-- @@
-- 审核阶段：系审核、科研处审核、专家盲审
CREATE TYPE review_stage AS ENUM ('department', 'research_office', 'expert');

-- @@
-- 审核决策：待审核、通过、不通过、退回
CREATE TYPE review_decision AS ENUM ('pending', 'approved', 'rejected', 'returned');

-- @@
-- 预期成果类型：阶段成果、最终成果
CREATE TYPE output_kind AS ENUM ('stage', 'final');

-- @@
-- 预算科目：社会科学、自然科学
CREATE TYPE budget_family AS ENUM ('social', 'natural');

-- @@
-- 附件类型：签名、支撑材料、其他
CREATE TYPE attachment_kind AS ENUM ('signature', 'supporting', 'other');

-- @@
-- 通知类型：状态变化、审核待办、截止提醒、系统通知
CREATE TYPE notification_kind AS ENUM ('status_changed', 'review_todo', 'deadline_reminder', 'system');

-- @@
-- 部门表
CREATE TABLE departments (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    name varchar(100) NOT NULL,
    code varchar(32),
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    CONSTRAINT departments_name_unique UNIQUE (name),
    CONSTRAINT departments_code_unique UNIQUE (code)
);

-- @@
-- 用户表：账号与申报书所需的个人资料
CREATE TABLE users (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    username varchar(64) NOT NULL,
    password_hash text NOT NULL,
    real_name varchar(64) NOT NULL,
    employee_no varchar(32),
    gender varchar(8),
    ethnicity varchar(32),
    birth_date date,
    admin_title varchar(64),
    professional_title varchar(64),
    research_expertise varchar(120),
    education varchar(32),
    degree varchar(32),
    phone varchar(20),
    email varchar(120),
    department_id uuid REFERENCES departments (id) ON DELETE RESTRICT,
    is_active boolean NOT NULL DEFAULT true,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    CONSTRAINT users_username_unique UNIQUE (username),
    CONSTRAINT users_employee_no_unique UNIQUE (employee_no),
    CONSTRAINT users_email_unique UNIQUE (email)
);

-- @@
-- 用户角色表：一个用户可有多个角色
CREATE TABLE user_roles (
    user_id uuid NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    role user_role NOT NULL,
    PRIMARY KEY (user_id, role)
);

-- @@
-- 申报批次表
CREATE TABLE application_cycles (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    cycle_year integer NOT NULL,
    title varchar(120) NOT NULL,
    opens_at timestamptz,
    closes_at timestamptz,
    status cycle_status NOT NULL DEFAULT 'draft',
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    CONSTRAINT application_cycles_year_title_unique UNIQUE (cycle_year, title),
    CONSTRAINT application_cycles_window CHECK (
        opens_at IS NULL OR closes_at IS NULL OR closes_at >= opens_at
    )
);

-- @@
-- 申报主表：封面、负责人快照、承诺、流程状态、立项结论
CREATE TABLE applications (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    cycle_id uuid NOT NULL REFERENCES application_cycles (id) ON DELETE RESTRICT,
    project_class project_class NOT NULL,
    code varchar(32),
    title varchar(40),
    keywords text,
    principal_id uuid NOT NULL REFERENCES users (id) ON DELETE RESTRICT,
    principal_name varchar(64),
    gender varchar(8),
    ethnicity varchar(32),
    birth_date date,
    admin_title varchar(64),
    professional_title varchar(64),
    research_expertise varchar(120),
    education varchar(32),
    degree varchar(32),
    mobile_phone varchar(20),
    contact_phone varchar(20),
    department_id uuid REFERENCES departments (id) ON DELETE RESTRICT,
    filled_on date,
    status application_status NOT NULL DEFAULT 'draft',
    commitment_accepted boolean NOT NULL DEFAULT false,
    commitment_accepted_at timestamptz,
    submitted_at timestamptz,
    concluded_by uuid REFERENCES users (id) ON DELETE RESTRICT,
    concluded_at timestamptz,
    conclusion_opinion text,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    CONSTRAINT applications_code_unique UNIQUE (code),
    CONSTRAINT applications_title_len CHECK (title IS NULL OR char_length(title) <= 40),
    CONSTRAINT applications_commitment_time CHECK (
        NOT commitment_accepted OR commitment_accepted_at IS NOT NULL
    ),
    CONSTRAINT applications_submitted_complete CHECK (
        status = 'draft' OR (
            title IS NOT NULL AND btrim(title) <> ''
            AND principal_name IS NOT NULL AND btrim(principal_name) <> ''
            AND department_id IS NOT NULL
            AND commitment_accepted
            AND submitted_at IS NOT NULL
        )
    ),
    CONSTRAINT applications_code_after_office CHECK (
        code IS NULL OR status IN ('office_approved', 'expert_reviewing', 'expert_rejected', 'established')
    ),
    CONSTRAINT applications_concluded CHECK (
        (status IN ('established', 'expert_rejected'))
        = (concluded_by IS NOT NULL AND concluded_at IS NOT NULL)
    )
);

-- @@
-- 主要参加者表：序号 1–5 且同一申报书内唯一，因此最多 5 人
CREATE TABLE application_members (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    application_id uuid NOT NULL REFERENCES applications (id) ON DELETE CASCADE,
    sort_order smallint NOT NULL,
    name varchar(64) NOT NULL,
    birth_date date,
    professional_title varchar(64),
    research_expertise varchar(120),
    education varchar(32),
    degree varchar(32),
    organization varchar(120),
    user_id uuid REFERENCES users (id) ON DELETE SET NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    CONSTRAINT application_members_sort_unique UNIQUE (application_id, sort_order),
    CONSTRAINT application_members_sort_range CHECK (sort_order BETWEEN 1 AND 5)
);

-- @@
-- 推荐人表：与申报书 1:1
CREATE TABLE recommenders (
    application_id uuid PRIMARY KEY REFERENCES applications (id) ON DELETE CASCADE,
    name varchar(64) NOT NULL,
    professional_title varchar(64) NOT NULL,
    expertise varchar(120) NOT NULL,
    organization varchar(120) NOT NULL,
    opinion text,
    signed_on date,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now()
);

-- @@
-- 专家指派表：同一专家不能重复指派同一申报书
CREATE TABLE expert_assignments (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    application_id uuid NOT NULL REFERENCES applications (id) ON DELETE CASCADE,
    expert_id uuid NOT NULL REFERENCES users (id) ON DELETE RESTRICT,
    assigned_by uuid NOT NULL REFERENCES users (id) ON DELETE RESTRICT,
    assigned_at timestamptz NOT NULL DEFAULT now(),
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    CONSTRAINT expert_assignments_unique UNIQUE (application_id, expert_id),
    CONSTRAINT expert_assignments_not_self CHECK (expert_id <> assigned_by)
);

-- @@
-- 审核表：系审核、科研处审核、专家盲审；专家记录关联指派并可评分
CREATE TABLE reviews (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    application_id uuid NOT NULL REFERENCES applications (id) ON DELETE CASCADE,
    stage review_stage NOT NULL,
    reviewer_id uuid NOT NULL REFERENCES users (id) ON DELETE RESTRICT,
    assignment_id uuid REFERENCES expert_assignments (id) ON DELETE CASCADE,
    decision review_decision NOT NULL DEFAULT 'pending',
    opinion text,
    score numeric(5, 2),
    facts_accurate boolean,
    team_qualified boolean,
    resources_ready boolean,
    agree_to_manage boolean,
    is_blind boolean NOT NULL DEFAULT false,
    reviewed_at timestamptz,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    CONSTRAINT reviews_assignment_unique UNIQUE (assignment_id),
    CONSTRAINT reviews_blind_iff_expert CHECK (is_blind = (stage = 'expert')),
    CONSTRAINT reviews_assignment_iff_expert CHECK ((assignment_id IS NOT NULL) = (stage = 'expert')),
    CONSTRAINT reviews_score_expert_only CHECK (score IS NULL OR stage = 'expert'),
    CONSTRAINT reviews_score_range CHECK (score IS NULL OR score BETWEEN 0 AND 100),
    CONSTRAINT reviews_department_checks_only CHECK (
        stage = 'department' OR (
            facts_accurate IS NULL AND team_qualified IS NULL
            AND resources_ready IS NULL AND agree_to_manage IS NULL
        )
    ),
    CONSTRAINT reviews_reviewed_at CHECK ((decision = 'pending') = (reviewed_at IS NULL)),
    CONSTRAINT reviews_opinion_required CHECK (
        decision NOT IN ('rejected', 'returned') OR (opinion IS NOT NULL AND btrim(opinion) <> '')
    ),
    CONSTRAINT reviews_expert_no_return CHECK (stage <> 'expert' OR decision <> 'returned')
);

-- @@
-- 申报附件表：文件本体在对象存储或本地目录，storage_key 指向
CREATE TABLE application_attachments (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    application_id uuid NOT NULL REFERENCES applications (id) ON DELETE CASCADE,
    kind attachment_kind NOT NULL DEFAULT 'supporting',
    file_name varchar(255) NOT NULL,
    content_type varchar(120) NOT NULL,
    byte_size bigint NOT NULL,
    storage_key text NOT NULL,
    uploaded_by uuid NOT NULL REFERENCES users (id) ON DELETE RESTRICT,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    CONSTRAINT application_attachments_size CHECK (byte_size BETWEEN 0 AND 20971520),
    CONSTRAINT application_attachments_storage_key_unique UNIQUE (storage_key)
);

-- @@
-- 社会科学 · 项目设计论证
CREATE TABLE ss_designs (
    application_id uuid PRIMARY KEY REFERENCES applications (id) ON DELETE CASCADE,
    topic_basis text,
    research_content text,
    methodology text,
    innovation text,
    expected_outcome text,
    reference_list text,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now()
);

-- @@
-- 社会科学 · 研究基础和条件保障
CREATE TABLE ss_foundations (
    application_id uuid PRIMARY KEY REFERENCES applications (id) ON DELETE CASCADE,
    academic_resume text,
    research_basis text,
    undertaken_projects text,
    support_conditions text,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now()
);

-- @@
-- 社会科学 · 近五年相关研究成果
CREATE TABLE ss_prior_achievements (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    application_id uuid NOT NULL REFERENCES applications (id) ON DELETE CASCADE,
    sort_order smallint NOT NULL,
    title varchar(200) NOT NULL,
    researchers varchar(120),
    outcome_form varchar(64),
    publication text,
    published_on varchar(32),
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    CONSTRAINT ss_prior_achievements_sort_unique UNIQUE (application_id, sort_order),
    CONSTRAINT ss_prior_achievements_sort_range CHECK (sort_order >= 1)
);

-- @@
-- 社会科学 · 预期研究成果：阶段性序号 1–10、最终序号 1–2，配合唯一约束限制数量
CREATE TABLE ss_expected_outputs (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    application_id uuid NOT NULL REFERENCES applications (id) ON DELETE CASCADE,
    kind output_kind NOT NULL,
    sort_order smallint NOT NULL,
    complete_on varchar(32),
    title varchar(200) NOT NULL,
    outcome_form varchar(64),
    owner_name varchar(64),
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    CONSTRAINT ss_expected_outputs_sort_unique UNIQUE (application_id, kind, sort_order),
    CONSTRAINT ss_expected_outputs_sort_range CHECK (
        sort_order >= 1 AND sort_order <= CASE kind WHEN 'stage' THEN 10 ELSE 2 END
    )
);

-- @@
-- 自然科学 · 项目简介
CREATE TABLE ns_profiles (
    application_id uuid PRIMARY KEY REFERENCES applications (id) ON DELETE CASCADE,
    summary text,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now()
);

-- @@
-- 自然科学 · 主要研究内容
CREATE TABLE ns_research_contents (
    application_id uuid PRIMARY KEY REFERENCES applications (id) ON DELETE CASCADE,
    content text,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now()
);

-- @@
-- 自然科学 · 主要技术经济指标
CREATE TABLE ns_indicators (
    application_id uuid PRIMARY KEY REFERENCES applications (id) ON DELETE CASCADE,
    paper_international integer NOT NULL DEFAULT 0,
    paper_domestic integer NOT NULL DEFAULT 0,
    paper_provincial integer NOT NULL DEFAULT 0,
    paper_core integer NOT NULL DEFAULT 0,
    paper_general integer NOT NULL DEFAULT 0,
    report_tech integer NOT NULL DEFAULT 0,
    report_research integer NOT NULL DEFAULT 0,
    report_transfer integer NOT NULL DEFAULT 0,
    report_other integer NOT NULL DEFAULT 0,
    monograph integer NOT NULL DEFAULT 0,
    patent_invention_applied integer NOT NULL DEFAULT 0,
    patent_invention_granted integer NOT NULL DEFAULT 0,
    patent_utility_applied integer NOT NULL DEFAULT 0,
    patent_utility_granted integer NOT NULL DEFAULT 0,
    patent_design_applied integer NOT NULL DEFAULT 0,
    patent_design_granted integer NOT NULL DEFAULT 0,
    patent_pct_applied integer NOT NULL DEFAULT 0,
    patent_pct_granted integer NOT NULL DEFAULT 0,
    patent_other_applied integer NOT NULL DEFAULT 0,
    patent_other_granted integer NOT NULL DEFAULT 0,
    software_copyright_applied integer NOT NULL DEFAULT 0,
    software_copyright_registered integer NOT NULL DEFAULT 0,
    ic_layout_applied integer NOT NULL DEFAULT 0,
    ic_layout_registered integer NOT NULL DEFAULT 0,
    plant_variety_applied integer NOT NULL DEFAULT 0,
    plant_variety_registered integer NOT NULL DEFAULT 0,
    standard_national_lead integer NOT NULL DEFAULT 0,
    standard_national_participate integer NOT NULL DEFAULT 0,
    standard_industry_lead integer NOT NULL DEFAULT 0,
    standard_industry_participate integer NOT NULL DEFAULT 0,
    standard_local_lead integer NOT NULL DEFAULT 0,
    standard_local_participate integer NOT NULL DEFAULT 0,
    standard_enterprise_lead integer NOT NULL DEFAULT 0,
    standard_enterprise_participate integer NOT NULL DEFAULT 0,
    pilot_line integer NOT NULL DEFAULT 0,
    production_line integer NOT NULL DEFAULT 0,
    new_sales_income numeric(14, 2) NOT NULL DEFAULT 0,
    new_profit_tax numeric(14, 2) NOT NULL DEFAULT 0,
    talent_train_senior integer NOT NULL DEFAULT 0,
    talent_train_intermediate integer NOT NULL DEFAULT 0,
    talent_introduce_senior integer NOT NULL DEFAULT 0,
    new_product integer NOT NULL DEFAULT 0,
    new_technology integer NOT NULL DEFAULT 0,
    new_process integer NOT NULL DEFAULT 0,
    new_material integer NOT NULL DEFAULT 0,
    support_platform integer NOT NULL DEFAULT 0,
    test_base integer NOT NULL DEFAULT 0,
    demo_site integer NOT NULL DEFAULT 0,
    benefit_note text,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    CONSTRAINT ns_indicators_money_nonnegative CHECK (
        new_sales_income >= 0 AND new_profit_tax >= 0
    )
);

-- @@
-- 自然科学 · 计划进度及目标
CREATE TABLE ns_schedules (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    application_id uuid NOT NULL REFERENCES applications (id) ON DELETE CASCADE,
    sort_order smallint NOT NULL,
    start_date date,
    end_date date,
    goal text,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    CONSTRAINT ns_schedules_sort_unique UNIQUE (application_id, sort_order),
    CONSTRAINT ns_schedules_sort_range CHECK (sort_order >= 1),
    CONSTRAINT ns_schedules_window CHECK (
        start_date IS NULL OR end_date IS NULL OR end_date >= start_date
    )
);

-- @@
-- 自然科学 · 仪器设备购买清单（单价 5000 元以上）
CREATE TABLE ns_equipments (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    application_id uuid NOT NULL REFERENCES applications (id) ON DELETE CASCADE,
    sort_order smallint NOT NULL,
    name_spec varchar(200) NOT NULL,
    quantity integer,
    unit_price numeric(14, 2),
    amount numeric(14, 2),
    funding_source varchar(64),
    purpose text,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    CONSTRAINT ns_equipments_sort_unique UNIQUE (application_id, sort_order),
    CONSTRAINT ns_equipments_sort_range CHECK (sort_order >= 1),
    CONSTRAINT ns_equipments_quantity CHECK (quantity IS NULL OR quantity > 0),
    CONSTRAINT ns_equipments_threshold CHECK (unit_price IS NULL OR unit_price >= 5000),
    CONSTRAINT ns_equipments_amount CHECK (amount IS NULL OR amount >= 0)
);

-- @@
-- 经费科目字典：applies_to 区分社科 / 自科，parent_code 表示层级，fillable=false 为汇总行
CREATE TABLE budget_subjects (
    code varchar(64) PRIMARY KEY,
    applies_to budget_family NOT NULL,
    group_name varchar(16) NOT NULL,
    parent_code varchar(64) REFERENCES budget_subjects (code) ON DELETE RESTRICT,
    name varchar(120) NOT NULL,
    sort_order integer NOT NULL,
    fillable boolean NOT NULL DEFAULT true,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    CONSTRAINT budget_subjects_group CHECK (group_name IN ('direct', 'indirect', 'total')),
    CONSTRAINT budget_subjects_sort_unique UNIQUE (applies_to, sort_order)
);

-- @@
-- 经费科目种子数据
INSERT INTO budget_subjects (code, applies_to, group_name, parent_code, name, sort_order, fillable)
VALUES
    ('ss_direct', 'social', 'direct', NULL, '一、直接费用', 10, false),
    ('ss_books', 'social', 'direct', 'ss_direct', '1.图书资料费', 11, true),
    ('ss_data', 'social', 'direct', 'ss_direct', '2.数据采集费', 12, true),
    ('ss_travel', 'social', 'direct', 'ss_direct', '3.调研差旅费', 13, true),
    ('ss_equipment', 'social', 'direct', 'ss_direct', '4.设备购置和使用费', 14, true),
    ('ss_publish', 'social', 'direct', 'ss_direct', '5.出版/文献/信息传播费', 15, true),
    ('ss_meeting', 'social', 'direct', 'ss_direct', '6.会议费', 16, true),
    ('ss_consult', 'social', 'direct', 'ss_direct', '7.咨询费', 17, true),
    ('ss_labor', 'social', 'direct', 'ss_direct', '8.劳务费', 18, true),
    ('ss_print', 'social', 'direct', 'ss_direct', '9.印刷费', 19, true),
    ('ss_other', 'social', 'direct', 'ss_direct', '10.其他（含培训费）', 20, true),
    ('ss_indirect', 'social', 'indirect', NULL, '二、间接费用', 30, true),
    ('ss_performance', 'social', 'indirect', 'ss_indirect', '其中：绩效支出', 31, true),
    ('ss_total', 'social', 'total', NULL, '经费合计', 40, false),
    ('ns_direct', 'natural', 'direct', NULL, '一、直接费用', 10, false),
    ('ns_equipment', 'natural', 'direct', 'ns_direct', '1.设备费', 11, false),
    ('ns_equipment_purchase', 'natural', 'direct', 'ns_equipment', '（1）购置设备费', 12, true),
    ('ns_equipment_trial', 'natural', 'direct', 'ns_equipment', '（2）试制设备费', 13, true),
    ('ns_equipment_retrofit', 'natural', 'direct', 'ns_equipment', '（3）设备改造与租赁费', 14, true),
    ('ns_material', 'natural', 'direct', 'ns_direct', '2.材料费', 15, true),
    ('ns_test', 'natural', 'direct', 'ns_direct', '3.测试化验加工费', 16, true),
    ('ns_fuel', 'natural', 'direct', 'ns_direct', '4.燃料动力费', 17, true),
    ('ns_travel', 'natural', 'direct', 'ns_direct', '5.差旅费/会议费/国际合作与交流费', 18, true),
    ('ns_publish', 'natural', 'direct', 'ns_direct', '6.出版/文献/信息传播/知识产权事务费', 19, true),
    ('ns_labor', 'natural', 'direct', 'ns_direct', '7.劳务费', 20, true),
    ('ns_consult', 'natural', 'direct', 'ns_direct', '8.专家咨询费', 21, true),
    ('ns_other', 'natural', 'direct', 'ns_direct', '9.其他（含培训费）', 22, true),
    ('ns_indirect', 'natural', 'indirect', NULL, '二、间接费用', 30, true),
    ('ns_performance', 'natural', 'indirect', 'ns_indirect', '其中：绩效支出', 31, true),
    ('ns_total', 'natural', 'total', NULL, '经费合计', 40, false);

-- @@
-- 申报预算表：申报书 × 科目金额（万元）
CREATE TABLE application_budgets (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    application_id uuid NOT NULL REFERENCES applications (id) ON DELETE CASCADE,
    subject_code varchar(64) NOT NULL REFERENCES budget_subjects (code) ON DELETE RESTRICT,
    amount numeric(14, 2) NOT NULL DEFAULT 0,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    CONSTRAINT application_budgets_unique UNIQUE (application_id, subject_code),
    CONSTRAINT application_budgets_amount_nonnegative CHECK (amount >= 0)
);

-- @@
-- 站内通知表
CREATE TABLE notifications (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    recipient_id uuid NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    application_id uuid REFERENCES applications (id) ON DELETE CASCADE,
    kind notification_kind NOT NULL,
    title varchar(120) NOT NULL,
    body text,
    read_at timestamptz,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now()
);

-- @@
-- 操作审计表：只追加，不修改
CREATE TABLE audit_logs (
    id bigint GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    actor_id uuid REFERENCES users (id) ON DELETE SET NULL,
    action varchar(64) NOT NULL,
    target_type varchar(64) NOT NULL,
    target_id uuid,
    detail jsonb,
    ip_address varchar(45),
    user_agent text,
    created_at timestamptz NOT NULL DEFAULT now()
);

-- @@
CREATE INDEX idx_users_department ON users (department_id);

-- @@
CREATE INDEX idx_applications_principal ON applications (principal_id);

-- @@
CREATE INDEX idx_applications_department_status ON applications (department_id, status);

-- @@
CREATE INDEX idx_applications_cycle_class ON applications (cycle_id, project_class);

-- @@
CREATE INDEX idx_application_members_user ON application_members (user_id);

-- @@
CREATE INDEX idx_reviews_application_stage ON reviews (application_id, stage);

-- @@
CREATE INDEX idx_reviews_reviewer ON reviews (reviewer_id);

-- @@
CREATE INDEX idx_expert_assignments_expert ON expert_assignments (expert_id);

-- @@
CREATE INDEX idx_application_attachments_application ON application_attachments (application_id);

-- @@
CREATE INDEX idx_notifications_recipient_unread ON notifications (recipient_id, created_at DESC) WHERE read_at IS NULL;

-- @@
CREATE INDEX idx_audit_logs_actor ON audit_logs (actor_id, created_at DESC);

-- @@
CREATE INDEX idx_audit_logs_target ON audit_logs (target_type, target_id);

-- @@
-- 申报批次种子数据
INSERT INTO application_cycles (cycle_year, title, status)
VALUES (2026, '2026年院级科研项目申报', 'draft');

-- @@
COMMENT ON TABLE applications IS '申报主表。封面、负责人快照、流程状态与立项结论。编号由科研处填写。';

-- @@
COMMENT ON COLUMN applications.code IS '封面编号，申请人不填，科研处审核通过后填写';

-- @@
COMMENT ON COLUMN applications.keywords IS '最多 5 个，中英文分号分隔';

-- @@
COMMENT ON COLUMN applications.project_class IS 'social=B.社会科学类；natural=A.自然科学类；dual_high=D.双高校建设专项';

-- @@
COMMENT ON COLUMN applications.concluded_by IS '作出立项 / 不予立项结论的科研处人员';

-- @@
COMMENT ON COLUMN applications.conclusion_opinion IS '立项 / 不予立项说明，对申请人可见';

-- @@
COMMENT ON TABLE reviews IS '系审核、科研处审核、专家盲审。专家记录的 reviewer_id 不得返回给申请人。';

-- @@
COMMENT ON COLUMN reviews.assignment_id IS '专家盲审对应的指派记录，其他阶段为空';

-- @@
COMMENT ON COLUMN reviews.score IS '专家盲审评分，0–100，仅专家阶段可填';

-- @@
COMMENT ON COLUMN reviews.facts_accurate IS '系审：申报内容是否属实';

-- @@
COMMENT ON COLUMN reviews.team_qualified IS '系审：负责人及参加者是否适合承担本项目';

-- @@
COMMENT ON COLUMN reviews.resources_ready IS '系审：本部门能否提供时间和条件';

-- @@
COMMENT ON COLUMN reviews.agree_to_manage IS '系审：是否同意承担管理任务和信誉保证';

-- @@
COMMENT ON COLUMN ns_indicators.new_sales_income IS '累计新增销售收入，单位万元';

-- @@
COMMENT ON COLUMN ns_indicators.new_profit_tax IS '累计新增利税，单位万元';

-- @@
COMMENT ON TABLE ns_equipments IS '单价 5000 元以上的仪器设备';

-- @@
COMMENT ON TABLE audit_logs IS '登录、导出、状态变更等操作审计，只追加不修改';

export type FormSection = {
  index: string;
  title: string;
  table: string;
  points: string[];
};

export const socialSections: FormSection[] = [
  {
    index: "封面",
    title: "项目封面",
    table: "applications",
    points: [
      "项目名称，最多 40 个字",
      "项目类别：B.社会科学类",
      "项目负责人、联系电话、所在部门、填表日期",
      "编号由科研处填写",
    ],
  },
  {
    index: "承诺",
    title: "申请者承诺",
    table: "applications",
    points: ["确认内容真实、无知识产权争议", "同意学院使用和宣传项目成果"],
  },
  {
    index: "一",
    title: "基本情况",
    table: "applications / application_members",
    points: [
      "关键词最多 5 个，分号分隔",
      "负责人的性别、民族、出生年月、行政职务、专业职务、研究专长、学历、学位、手机",
      "主要参加者最多 5 人，不含负责人",
    ],
  },
  {
    index: "二",
    title: "近五年相关研究成果",
    table: "ss_prior_achievements",
    points: ["成果名称、研究者、成果形式、发表或交流情况、发表出版时间"],
  },
  {
    index: "三",
    title: "项目设计论证",
    table: "ss_designs",
    points: [
      "选题依据",
      "研究内容",
      "思路方法",
      "创新之处",
      "预期成果",
      "参考文献",
    ],
  },
  {
    index: "四",
    title: "研究基础和条件保障",
    table: "ss_foundations",
    points: ["学术简历", "研究基础", "承担项目", "条件保障"],
  },
  {
    index: "五",
    title: "预期研究成果",
    table: "ss_expected_outputs",
    points: ["主要阶段性成果最多 10 项", "最终研究成果最多 2 项"],
  },
  {
    index: "六",
    title: "经费预算",
    table: "application_budgets",
    points: [
      "直接费用 10 项，含图书资料、数据采集、调研差旅等",
      "间接费用及其中的绩效支出",
    ],
  },
  {
    index: "七",
    title: "推荐人意见",
    table: "recommenders",
    points: [
      "不具备中级及以上职称或研究生及以上学历时须推荐",
      "推荐人不能是负责人或参加者",
    ],
  },
];

export const naturalSections: FormSection[] = [
  {
    index: "封面",
    title: "项目封面",
    table: "applications",
    points: [
      "项目名称，最多 40 个字",
      "项目类别：A.自然科学类，或 D.双高校建设专项",
      "项目负责人、联系电话、所在部门、填表日期",
    ],
  },
  {
    index: "承诺",
    title: "申请者承诺",
    table: "applications",
    points: ["确认内容真实、无知识产权争议", "同意学院使用和宣传项目成果"],
  },
  {
    index: "一",
    title: "基本情况与项目简介",
    table: "applications / ns_profiles",
    points: [
      "关键词最多 5 个",
      "负责人信息与主要参加者，参加者最多 5 人",
      "项目简介：主要研究内容与技术经济指标",
    ],
  },
  {
    index: "二",
    title: "主要研究内容",
    table: "ns_research_contents",
    points: ["目标与任务", "技术难点", "关键技术与创新点"],
  },
  {
    index: "三",
    title: "主要技术经济指标",
    table: "ns_indicators",
    points: [
      "论文、报告、专著、知识产权、标准",
      "中试、生产线、销售收入与利税",
      "人才培养、新产品新技术、平台与示范点",
      "社会效益说明",
    ],
  },
  {
    index: "四",
    title: "计划进度及目标",
    table: "ns_schedules",
    points: ["开始日期、结束日期、阶段目标"],
  },
  {
    index: "五",
    title: "仪器设备购买清单",
    table: "ns_equipments",
    points: ["单价 5000 元以上", "名称规格、数量、单价、金额、资金来源、用途"],
  },
  {
    index: "六",
    title: "经费预算",
    table: "application_budgets",
    points: [
      "设备费分购置、试制、改造与租赁",
      "材料、测试化验、燃料动力、差旅会议、知识产权、劳务、专家咨询",
      "间接费用及绩效支出",
    ],
  },
  {
    index: "七",
    title: "推荐人意见",
    table: "recommenders",
    points: ["职称或学历未达要求时须同行专家推荐", "推荐人不能是负责人或参加者"],
  },
];

export const departmentChecks = [
  "申报书所填写的内容是否属实",
  "负责人及参加者是否适合承担本项目",
  "本部门能否提供完成项目所需的时间和条件",
  "本部门是否同意承担管理任务和信誉保证",
];

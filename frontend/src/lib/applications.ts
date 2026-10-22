import { apiSend } from "@/lib/api";

export type ProjectClassCode = "social" | "natural" | "dual_high";

export type MineItem = {
  id: string;
  project_class: ProjectClassCode;
  title: string | null;
  status: string;
  updated_at: string;
  submitted_at: string | null;
};

export type MemberInput = {
  name: string;
  birth_date: string;
  professional_title: string;
  research_expertise: string;
  education: string;
  degree: string;
  organization: string;
};

export type BudgetLine = {
  code: string;
  name: string;
  group_name: string;
  parent_code: string | null;
  fillable: boolean;
  amount: string;
};

export type TextRow = {
  title: string;
  researchers: string;
  outcome_form: string;
  publication: string;
  published_on: string;
};

export type OutputRow = {
  complete_on: string;
  title: string;
  outcome_form: string;
  owner_name: string;
};

export type ScheduleRow = {
  start_date: string;
  end_date: string;
  goal: string;
};

export type EquipmentRow = {
  name_spec: string;
  quantity: string;
  unit_price: string;
  funding_source: string;
  purpose: string;
};

export type IndicatorInput = {
  paper_core: number;
  paper_general: number;
  paper_provincial: number;
  paper_domestic: number;
  paper_international: number;
  report_research: number;
  monograph: number;
  software_copyright_applied: number;
  patent_invention_applied: number;
  patent_utility_applied: number;
  new_product: number;
  new_technology: number;
  demo_site: number;
  test_base: number;
  new_sales_income: string;
  new_profit_tax: string;
  benefit_note: string;
};

export type ApplicationDetail = {
  id: string;
  project_class: ProjectClassCode;
  title: string | null;
  keywords: string | null;
  status: string;
  editable: boolean;
  principal_name: string | null;
  gender: string | null;
  ethnicity: string | null;
  birth_date: string | null;
  admin_title: string | null;
  professional_title: string | null;
  research_expertise: string | null;
  education: string | null;
  degree: string | null;
  mobile_phone: string | null;
  department_name: string | null;
  filled_on: string | null;
  commitment_accepted: boolean;
  members: Array<{
    name: string;
    birth_date: string | null;
    professional_title: string | null;
    research_expertise: string | null;
    education: string | null;
    degree: string | null;
    organization: string | null;
  }>;
  recommender: {
    name: string;
    professional_title: string;
    expertise: string;
    organization: string;
    opinion: string | null;
    signed_on: string | null;
  } | null;
  budget: {
    lines: BudgetLine[];
    direct_total: string;
    indirect_total: string;
    total: string;
  };
  ss_achievements: Array<{
    title: string;
    researchers: string | null;
    outcome_form: string | null;
    publication: string | null;
    published_on: string | null;
  }>;
  ss_design: {
    topic_basis: string | null;
    research_content: string | null;
    methodology: string | null;
    innovation: string | null;
    expected_outcome: string | null;
    reference_list: string | null;
  };
  ss_foundation: {
    academic_resume: string | null;
    research_basis: string | null;
    undertaken_projects: string | null;
    support_conditions: string | null;
  };
  ss_stage_outputs: OutputRow[];
  ss_final_outputs: OutputRow[];
  ns_profile: string | null;
  ns_content: string | null;
  ns_indicators: IndicatorInput & Record<string, number | string | null>;
  ns_schedules: Array<{ start_date: string | null; end_date: string | null; goal: string | null }>;
  ns_equipments: Array<{
    name_spec: string;
    quantity: number;
    unit_price: string;
    amount: string;
    funding_source: string | null;
    purpose: string | null;
  }>;
};

export const statusLabels: Record<string, string> = {
  draft: "草稿",
  submitted: "待系审核",
  dept_returned: "系部退回",
  dept_rejected: "系部不通过",
  dept_approved: "待科研处审核",
  office_returned: "科研处退回",
  office_rejected: "科研处不通过",
  office_approved: "待指派专家",
  expert_reviewing: "专家盲审",
  expert_rejected: "不予立项",
  established: "已立项",
};

export const classLabels: Record<string, string> = {
  social: "社会科学",
  natural: "自然科学",
  dual_high: "双高专项",
};

export async function fetchMine(): Promise<MineItem[]> {
  const body = await apiSend<{ items: MineItem[] }>("/api/applications/mine");
  return body.items;
}

export function createApplication(projectClass: ProjectClassCode): Promise<ApplicationDetail> {
  return apiSend("/api/applications", {
    method: "POST",
    body: JSON.stringify({ project_class: projectClass }),
  });
}

export function fetchApplication(id: string): Promise<ApplicationDetail> {
  return apiSend(`/api/applications/${id}`);
}

export function saveSection(id: string, section: string, body: unknown): Promise<void> {
  return apiSend(`/api/applications/${id}/${section}`, {
    method: "PUT",
    body: JSON.stringify(body),
  });
}

export function submitApplication(id: string): Promise<ApplicationDetail> {
  return apiSend(`/api/applications/${id}/submit`, {
    method: "POST",
    body: JSON.stringify({ commitment_accepted: true }),
  });
}

export function withdrawApplication(id: string): Promise<ApplicationDetail> {
  return apiSend(`/api/applications/${id}/withdraw`, { method: "POST" });
}

export function deleteDraft(id: string): Promise<void> {
  return apiSend(`/api/applications/${id}`, { method: "DELETE" });
}

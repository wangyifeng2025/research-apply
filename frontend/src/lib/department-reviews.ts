import { apiSend } from "@/lib/api";
import type { ApplicationDetail } from "@/lib/applications";

export type DepartmentDecision = "approved" | "returned" | "rejected";

export type DepartmentReviewItem = {
  id: string;
  title: string | null;
  project_class: string;
  principal_name: string | null;
  professional_title: string | null;
  department_name: string;
  submitted_at: string | null;
  status: string;
  budget_total: string;
  decision: DepartmentDecision | null;
  opinion: string | null;
  reviewed_at: string | null;
  facts_accurate: boolean | null;
  team_qualified: boolean | null;
  resources_ready: boolean | null;
  agree_to_manage: boolean | null;
};

export type DepartmentReviewInput = {
  decision: DepartmentDecision;
  opinion: string;
  facts_accurate: boolean;
  team_qualified: boolean;
  resources_ready: boolean;
  agree_to_manage: boolean;
};

export async function fetchDepartmentQueue(scope: "pending" | "reviewed"): Promise<DepartmentReviewItem[]> {
  const body = await apiSend<{ items: DepartmentReviewItem[] }>(`/api/department-reviews?scope=${scope}`);
  return body.items;
}

export function fetchDepartmentReview(id: string): Promise<ApplicationDetail> {
  return apiSend(`/api/department-reviews/${id}`);
}

export function submitDepartmentReview(id: string, input: DepartmentReviewInput): Promise<DepartmentReviewItem> {
  return apiSend(`/api/department-reviews/${id}`, {
    method: "POST",
    body: JSON.stringify(input),
  });
}

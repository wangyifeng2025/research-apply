import { apiSend } from "@/lib/api";
import type { ApplicationDetail } from "@/lib/applications";

export type OfficeScope =
  | "pending"
  | "assign"
  | "reviewing"
  | "conclude"
  | "established"
  | "expert_rejected"
  | "returned"
  | "rejected";

export type OfficeReviewItem = {
  id: string;
  title: string | null;
  project_class: string;
  principal_name: string | null;
  professional_title: string | null;
  department_name: string;
  submitted_at: string | null;
  status: string;
  budget_total: string;
  code: string | null;
  assigned_count: number;
  reviewed_count: number;
  average_score: string | null;
  decision: "approved" | "returned" | "rejected" | null;
  opinion: string | null;
  reviewed_at: string | null;
  conclusion_opinion: string | null;
};

export type AssignmentProgress = {
  id: string;
  expert_id: string;
  expert_name: string;
  research_expertise: string | null;
  reviewed: boolean;
  decision: "approved" | "rejected" | null;
  opinion: string | null;
  score: string | null;
  reviewed_at: string | null;
};

export type OfficeDetail = {
  application: ApplicationDetail;
  code: string | null;
  conclusion_opinion: string | null;
  assignments: AssignmentProgress[];
  average_score: string | null;
  ready_to_conclude: boolean;
};

export type ExpertOption = {
  id: string;
  real_name: string;
  research_expertise: string | null;
  department_name: string;
};

export async function fetchOfficeQueue(scope: OfficeScope): Promise<OfficeReviewItem[]> {
  const body = await apiSend<{ items: OfficeReviewItem[] }>(`/api/office-reviews?scope=${scope}`);
  return body.items;
}

export function fetchOfficeReview(id: string): Promise<OfficeDetail> {
  return apiSend(`/api/office-reviews/${id}`);
}

export function submitOfficeReview(
  id: string,
  input: { decision: "approved" | "returned" | "rejected"; opinion: string; code: string },
): Promise<OfficeReviewItem> {
  return apiSend(`/api/office-reviews/${id}`, { method: "POST", body: JSON.stringify(input) });
}

export async function fetchExperts(query: string): Promise<ExpertOption[]> {
  const suffix = query.trim() ? `?q=${encodeURIComponent(query.trim())}` : "";
  const body = await apiSend<{ items: ExpertOption[] }>(`/api/office-reviews/experts${suffix}`);
  return body.items;
}

export function assignExperts(id: string, expertIds: string[]): Promise<OfficeDetail> {
  return apiSend(`/api/office-reviews/${id}/assignments`, {
    method: "POST",
    body: JSON.stringify({ expert_ids: expertIds }),
  });
}

export function revokeExpert(id: string, assignmentId: string): Promise<OfficeDetail> {
  return apiSend(`/api/office-reviews/${id}/assignments/${assignmentId}`, { method: "DELETE" });
}

export function concludeApplication(
  id: string,
  input: { decision: "established" | "expert_rejected"; opinion: string },
): Promise<OfficeReviewItem> {
  return apiSend(`/api/office-reviews/${id}/conclusion`, { method: "POST", body: JSON.stringify(input) });
}

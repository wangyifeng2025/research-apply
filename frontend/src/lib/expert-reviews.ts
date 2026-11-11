import { apiSend } from "@/lib/api";
import type { ApplicationDetail, AttachmentItem } from "@/lib/applications";

export type ExpertReviewItem = {
  id: string;
  project_class: string;
  title: string | null;
  decision: "approved" | "rejected" | null;
  score: string | null;
  opinion: string | null;
  reviewed_at: string | null;
};

export type BlindApplication = {
  id: string;
  project_class: string;
  title: string | null;
  keywords: string | null;
  budget: ApplicationDetail["budget"];
  ss_achievements: Array<{ title: string; outcome_form: string | null; publication: string | null; published_on: string | null }>;
  ss_design: ApplicationDetail["ss_design"];
  ss_foundation: {
    research_basis: string | null;
    undertaken_projects: string | null;
    support_conditions: string | null;
  };
  ss_stage_outputs: Array<{ complete_on: string | null; title: string; outcome_form: string | null }>;
  ss_final_outputs: Array<{ complete_on: string | null; title: string; outcome_form: string | null }>;
  ns_profile: string | null;
  ns_content: string | null;
  ns_indicators: ApplicationDetail["ns_indicators"];
  ns_schedules: ApplicationDetail["ns_schedules"];
  ns_equipments: Array<{
    name_spec: string;
    quantity: number;
    unit_price: string;
    amount: string;
    funding_source: string | null;
    purpose: string | null;
  }>;
  attachments: AttachmentItem[];
};

export type ExpertDetail = {
  application: BlindApplication;
  review: { decision: "approved" | "rejected"; score: string; opinion: string | null; reviewed_at: string | null } | null;
};

export async function fetchExpertQueue(scope: "pending" | "reviewed"): Promise<ExpertReviewItem[]> {
  const body = await apiSend<{ items: ExpertReviewItem[] }>(`/api/expert-reviews?scope=${scope}`);
  return body.items;
}

export function fetchExpertReview(id: string): Promise<ExpertDetail> {
  return apiSend(`/api/expert-reviews/${id}`);
}

export function submitExpertReview(
  id: string,
  input: { decision: "approved" | "rejected"; opinion: string; score: string },
): Promise<ExpertDetail["review"]> {
  return apiSend(`/api/expert-reviews/${id}`, { method: "POST", body: JSON.stringify(input) });
}

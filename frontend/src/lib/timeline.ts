import { apiSend } from "@/lib/api";

export type TimelineEvent = {
  stage: string;
  actor: string;
  decision: string;
  opinion: string | null;
  score: string | null;
  reviewed_at: string;
  facts_accurate: boolean | null;
  team_qualified: boolean | null;
  resources_ready: boolean | null;
  agree_to_manage: boolean | null;
};

export async function fetchTimeline(id: string): Promise<TimelineEvent[]> {
  const body = await apiSend<{ events: TimelineEvent[] }>(`/api/applications/${id}/timeline`);
  return body.events;
}

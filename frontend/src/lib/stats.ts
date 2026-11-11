import { ApiError, apiSend } from "@/lib/api";

export type StatsCycle = {
  id: string;
  cycle_year: number;
  title: string;
  status: string;
};

export type StatsBucket = {
  code: string;
  label: string;
  count: number;
  amount: string;
};

export type StatsOverview = {
  cycle: StatsCycle | null;
  cycles: StatsCycle[];
  total_count: number;
  total_amount: string;
  by_class: StatsBucket[];
  by_department: StatsBucket[];
  by_status: StatsBucket[];
};

export function fetchStats(cycleId?: string): Promise<StatsOverview> {
  const query = cycleId ? `?cycle_id=${encodeURIComponent(cycleId)}` : "";
  return apiSend(`/api/stats/overview${query}`);
}

export async function downloadStats(cycleId: string): Promise<void> {
  const response = await fetch(`/api/stats/export?cycle_id=${encodeURIComponent(cycleId)}`, {
    credentials: "include",
  });
  if (!response.ok) {
    const body = (await response.json().catch(() => null)) as { error?: string } | null;
    throw new ApiError(response.status, body?.error ?? "导出失败");
  }
  const blob = await response.blob();
  const url = URL.createObjectURL(blob);
  const link = document.createElement("a");
  link.href = url;
  link.download = "申报统计.xlsx";
  link.click();
  URL.revokeObjectURL(url);
}

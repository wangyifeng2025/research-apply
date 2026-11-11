"use client";

import { useEffect, useState } from "react";
import { useRouter } from "next/navigation";
import { useSyncExternalStore } from "react";

import { Icon } from "@/components/icon";
import { AdminPanel, AdminStat, messageOf } from "@/components/admin-ui";
import { Button } from "@/components/ui/button";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import { StickyTable, type StickyColumn } from "@/components/ui/sticky-table";
import { WorkspaceShell } from "@/components/workspace-shell";
import { readActiveLogin, roleHomes, type LoginRole } from "@/lib/login";
import { downloadStats, fetchStats, type StatsBucket, type StatsOverview } from "@/lib/stats";

const columns: StickyColumn<StatsBucket>[] = [
  { header: "名称", width: 280, cell: (row) => row.label },
  { header: "申报数量", width: 140, align: "right", cell: (row) => row.count.toLocaleString() },
  { header: "申请经费（万元）", width: 180, align: "right", cell: (row) => row.amount },
];

export function StatsPage() {
  const router = useRouter();
  const role = useSyncExternalStore(subscribeLogin, readShellRole, () => "office" as const);
  const [overview, setOverview] = useState<StatsOverview | null>(null);
  const [cycleId, setCycleId] = useState("");
  const [error, setError] = useState("");
  const [exporting, setExporting] = useState(false);

  useEffect(() => {
    let cancelled = false;
    async function load() {
      try {
        const next = await fetchStats(cycleId || undefined);
        if (cancelled) return;
        setOverview(next);
        setError("");
        if (!cycleId && next.cycle) setCycleId(next.cycle.id);
      } catch (loadError) {
        if (!cancelled) setError(messageOf(loadError));
      }
    }
    void load();
    return () => {
      cancelled = true;
    };
  }, [cycleId]);

  async function exportWorkbook() {
    if (!cycleId) return;
    setExporting(true);
    setError("");
    try {
      await downloadStats(cycleId);
    } catch (exportError) {
      setError(messageOf(exportError));
    } finally {
      setExporting(false);
    }
  }

  const shellRole = role;

  return (
    <WorkspaceShell
      role={shellRole}
      section={shellRole === "admin" ? "console" : "stats"}
      onSection={(id) => router.push(id === "stats" ? "/stats" : roleHomes[shellRole])}
      subtitle="申报统计"
    >
      <div className="flex flex-wrap items-end justify-between gap-3">
        <div>
          <h1 className="text-[20px] font-semibold text-ink-900">申报统计</h1>
          <p className="mt-1 text-[12.5px] text-ink-500">按批次查看类别、部门和状态的申报数量与申请经费。</p>
        </div>
        <div className="flex items-center gap-2">
          <Select value={cycleId || undefined} onValueChange={setCycleId} disabled={!overview?.cycles.length}>
            <SelectTrigger className="w-64">
              <SelectValue placeholder="选择批次" />
            </SelectTrigger>
            <SelectContent>
              {overview?.cycles.map((cycle) => (
                <SelectItem key={cycle.id} value={cycle.id}>
                  {cycle.title}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
          <Button variant="outline" disabled={!cycleId || exporting} onClick={() => void exportWorkbook()}>
            <Icon name="mdi:file-excel-outline" />
            {exporting ? "导出中" : "导出 Excel"}
          </Button>
        </div>
      </div>
      {error ? <p className="mt-4 text-[12.5px] text-cinnabar">{error}</p> : null}
      <div className="mt-5 grid gap-3 sm:grid-cols-2">
        <AdminStat label="申报数量" value={overview?.total_count ?? "—"} unit="项" icon="mdi:file-document-multiple-outline" />
        <AdminStat label="申请经费" value={overview?.total_amount ?? "—"} unit="万元" icon="mdi:cash-multiple" gold />
      </div>
      <div className="mt-5 grid gap-4 xl:grid-cols-3">
        <Bucket title="按类别" rows={overview?.by_class ?? []} />
        <Bucket title="按部门" rows={overview?.by_department ?? []} />
        <Bucket title="按状态" rows={overview?.by_status ?? []} />
      </div>
    </WorkspaceShell>
  );
}

function Bucket({ title, rows }: { title: string; rows: StatsBucket[] }) {
  return (
    <AdminPanel>
      <h2 className="border-b border-ink-100 px-4 py-3 text-[13px] font-medium text-ink-800">{title}</h2>
      <StickyTable columns={columns} rows={rows} rowKey={(row) => row.code || row.label} empty="暂无申报" />
    </AdminPanel>
  );
}

function subscribeLogin() {
  return () => undefined;
}

function readShellRole(): LoginRole {
  return readActiveLogin()?.role ?? "office";
}

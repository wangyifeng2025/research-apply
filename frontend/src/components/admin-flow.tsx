"use client";

import { useState, type FormEvent } from "react";

import { AdminPanel, Field, messageOf } from "@/components/admin-ui";
import { Icon } from "@/components/icon";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { DialogBody, DialogFooter, FormDialog } from "@/components/ui/dialog";
import { Input } from "@/components/ui/input";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import { StickyTable } from "@/components/ui/sticky-table";
import { createCycle, updateCycle, type Cycle, type CycleInput, type CycleStatus } from "@/lib/admin";
import type { WorkflowStep } from "@/lib/api";

type CycleForm = { year: string; title: string; opens: string; closes: string; status: CycleStatus };

export function AdminFlow({ cycles, workflow, onChanged, onNotify }: { cycles: Cycle[]; workflow: WorkflowStep[]; onChanged: () => Promise<void>; onNotify: (message: string) => void }) {
  const [busy, setBusy] = useState(false);
  const [editorOpen, setEditorOpen] = useState(false);
  const [editing, setEditing] = useState<Cycle | null>(null);
  const [form, setForm] = useState<CycleForm>({ year: "2026", title: "", opens: "", closes: "", status: "draft" });
  const [now] = useState(() => Date.now());
  const steps = workflow.length > 0 ? workflow : [
    { code: "teacher", label: "教师填报" },
    { code: "department", label: "系审核" },
    { code: "research_office", label: "科研处审核" },
    { code: "expert", label: "专家盲审" },
  ];

  function openEditor(cycle?: Cycle) {
    setEditing(cycle ?? null);
    setForm(cycle ? { year: String(cycle.cycle_year), title: cycle.title, opens: localInput(cycle.opens_at), closes: localInput(cycle.closes_at), status: cycle.status } : { year: String(new Date().getFullYear()), title: "", opens: "", closes: "", status: "draft" });
    setEditorOpen(true);
  }

  async function submitCycle(event: FormEvent) {
    event.preventDefault();
    setBusy(true);
    try {
      const input: CycleInput = {
        cycle_year: Number(form.year),
        title: form.title,
        opens_at: form.opens ? new Date(form.opens).toISOString() : null,
        closes_at: form.closes ? new Date(form.closes).toISOString() : null,
        status: form.status,
      };
      if (editing) await updateCycle(editing.id, input);
      else await createCycle(input);
      await onChanged();
      setEditorOpen(false);
      onNotify(editing ? "批次已更新" : "批次已创建");
    } catch (error) {
      onNotify(messageOf(error));
    } finally {
      setBusy(false);
    }
  }

  async function setStatus(cycle: Cycle, status: CycleStatus) {
    setBusy(true);
    try {
      await updateCycle(cycle.id, { status });
      await onChanged();
      onNotify(status === "open" ? "批次已开放" : status === "closed" ? "批次已关闭" : "批次已改为未开放");
    } catch (error) {
      onNotify(messageOf(error));
    } finally {
      setBusy(false);
    }
  }

  return (
    <div className="mt-4 space-y-4">
      <AdminPanel className="p-4 sm:p-5">
        <h2 className="text-[14px] font-semibold text-ink-900">申报流程</h2>
        <p className="mt-1 text-[12px] text-ink-500">环节顺序来自系统信息。节点时限和开关还没有配置接口，这里只展示，不能保存。</p>
        <ol className="mt-4 grid gap-2 sm:grid-cols-2 xl:grid-cols-4">
          {steps.map((step, index) => (
            <li key={step.code} className="flex items-center gap-3 rounded-lg border border-ink-200 px-3 py-2.5">
              <span className="flex h-7 w-7 shrink-0 items-center justify-center rounded-full bg-teal-700 text-[12px] font-semibold text-white">{index + 1}</span>
              <span className="text-[13px] font-medium text-ink-800">{step.label}</span>
            </li>
          ))}
        </ol>
      </AdminPanel>
      <AdminPanel className="overflow-hidden">
        <div className="flex items-center justify-between gap-3 border-b border-ink-100 px-4 py-3">
          <div>
            <h2 className="text-[14px] font-semibold text-ink-900">申报批次</h2>
            <p className="mt-0.5 text-[12px] text-ink-400">新建、修改时间和开放状态会写入批次接口</p>
          </div>
          <Button onClick={() => openEditor()}><Icon name="mdi:plus" />新建批次</Button>
        </div>
        <StickyTable
          rows={cycles}
          rowKey={(cycle) => cycle.id}
          empty="还没有申报批次"
          columns={[
            { header: "年度", width: 88, cell: (cycle) => <span className="font-medium text-ink-900">{cycle.cycle_year}</span> },
            { header: "批次名称", width: 220, cell: (cycle) => cycle.title },
            { header: "开放时间", width: 168, cell: (cycle) => formatDate(cycle.opens_at) },
            { header: "截止时间", width: 168, cell: (cycle) => formatDate(cycle.closes_at) },
            { header: "状态", width: 120, cell: (cycle) => <CycleStatusBadge cycle={cycle} now={now} /> },
            {
              header: "操作",
              width: 210,
              align: "right",
              cell: (cycle) => (
                <div className="flex justify-end gap-1">
                  <Button variant="subtle" size="sm" onClick={() => openEditor(cycle)}>编辑</Button>
                  {cycle.status !== "open" ? <Button variant="subtle" size="sm" disabled={busy} onClick={() => void setStatus(cycle, "open")}>开放</Button> : null}
                  {cycle.status !== "closed" ? <Button variant="warning" size="sm" disabled={busy} onClick={() => void setStatus(cycle, "closed")}>关闭</Button> : null}
                </div>
              ),
            },
          ]}
        />
      </AdminPanel>
      <FormDialog open={editorOpen} title={editing ? "编辑批次" : "新建批次"} description="时间留空表示不限制。教师首次提交时会校验所属批次。" onClose={() => setEditorOpen(false)}>
        <form className="flex min-h-0 flex-1 flex-col" onSubmit={(event) => void submitCycle(event)}>
          <DialogBody className="grid gap-3 sm:grid-cols-2">
            <Field label="年度" required><Input required type="number" value={form.year} onChange={(event) => setForm({ ...form, year: event.target.value })} /></Field>
            <Field label="状态">
              <Select value={form.status} onValueChange={(value) => setForm({ ...form, status: value as CycleStatus })}>
                <SelectTrigger><SelectValue /></SelectTrigger>
                <SelectContent>
                  <SelectItem value="draft">未开放</SelectItem>
                  <SelectItem value="open">申报中</SelectItem>
                  <SelectItem value="closed">已关闭</SelectItem>
                </SelectContent>
              </Select>
            </Field>
            <Field label="批次名称" required className="sm:col-span-2"><Input required value={form.title} onChange={(event) => setForm({ ...form, title: event.target.value })} /></Field>
            <Field label="开放时间"><Input type="datetime-local" value={form.opens} onChange={(event) => setForm({ ...form, opens: event.target.value })} /></Field>
            <Field label="截止时间"><Input type="datetime-local" value={form.closes} onChange={(event) => setForm({ ...form, closes: event.target.value })} /></Field>
          </DialogBody>
          <DialogFooter>
            <Button variant="outline" onClick={() => setEditorOpen(false)}>取消</Button>
            <Button type="submit" disabled={busy}>{busy ? "保存中…" : "保存批次"}</Button>
          </DialogFooter>
        </form>
      </FormDialog>
    </div>
  );
}

function CycleStatusBadge({ cycle, now }: { cycle: Cycle; now: number }) {
  let label = cycle.status === "open" ? "申报中" : cycle.status === "closed" ? "已关闭" : "未开放";
  let tone: "teal" | "neutral" | "gold" = label === "申报中" ? "teal" : "neutral";
  if (cycle.status === "open" && cycle.opens_at && new Date(cycle.opens_at).getTime() > now) {
    label = "未到开放时间";
    tone = "gold";
  }
  if (cycle.status === "open" && cycle.closes_at && new Date(cycle.closes_at).getTime() < now) {
    label = "已过截止时间";
    tone = "gold";
  }
  return <Badge tone={tone}>{label}</Badge>;
}

function localInput(value: string | null) {
  if (!value) return "";
  const date = new Date(value);
  return new Date(date.getTime() - date.getTimezoneOffset() * 60000).toISOString().slice(0, 16);
}

function formatDate(value: string | null) {
  return value ? new Intl.DateTimeFormat("zh-CN", { year: "numeric", month: "2-digit", day: "2-digit", hour: "2-digit", minute: "2-digit", hour12: false }).format(new Date(value)) : "不限";
}

"use client";

import { useEffect, useMemo, useState } from "react";

import { ApplicationReading } from "@/components/application-reading";
import { ReviewTimeline } from "@/components/review-timeline";
import { Icon } from "@/components/icon";
import { Button } from "@/components/ui/button";
import { Checkbox } from "@/components/ui/checkbox";
import { ConfirmDialog } from "@/components/ui/alert-dialog";
import {
  Dialog,
  DialogBody,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { Input } from "@/components/ui/input";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { StickyTable } from "@/components/ui/sticky-table";
import { Textarea } from "@/components/ui/textarea";
import { SideNav, WorkspaceShell } from "@/components/workspace-shell";
import { ApiError } from "@/lib/api";
import { classLabels, statusLabels } from "@/lib/applications";
import { fetchTimeline, type TimelineEvent } from "@/lib/timeline";
import {
  assignExperts,
  concludeApplication,
  fetchExperts,
  fetchOfficeQueue,
  fetchOfficeReview,
  revokeExpert,
  submitOfficeReview,
  type ExpertOption,
  type OfficeDetail,
  type OfficeReviewItem,
  type OfficeScope,
} from "@/lib/office-reviews";

const sections: Array<{ id: OfficeScope; label: string; icon: string }> = [
  { id: "pending", label: "待审核", icon: "mdi:inbox-arrow-down-outline" },
  { id: "assign", label: "待指派", icon: "mdi:account-supervisor-outline" },
  { id: "reviewing", label: "盲审中", icon: "mdi:gavel" },
  { id: "conclude", label: "待立项", icon: "mdi:clipboard-check-outline" },
  { id: "established", label: "拟立项", icon: "mdi:check-decagram-outline" },
  { id: "expert_rejected", label: "不予立项", icon: "mdi:file-alert-outline" },
  { id: "returned", label: "已退回", icon: "mdi:file-undo-outline" },
  { id: "rejected", label: "不通过", icon: "mdi:close" },
];

const emptyText: Record<OfficeScope, string> = {
  pending: "还没有待科研处审核的申报。",
  assign: "还没有等待指派专家的申报。",
  reviewing: "没有正在盲审的申报。",
  conclude: "没有专家已经全部提交、等待拟立项的申报。",
  established: "还没有拟立项的申报。",
  expert_rejected: "还没有不予立项的申报。",
  returned: "还没有退回的申报。",
  rejected: "还没有不通过的申报。",
};

export function OfficeWorkspace() {
  const [section, setSection] = useState<OfficeScope>("pending");
  const [queues, setQueues] = useState<
    Partial<Record<OfficeScope, OfficeReviewItem[]>>
  >({});
  const [loading, setLoading] = useState(true);
  const [version, setVersion] = useState(0);
  const [loadError, setLoadError] = useState("");
  const [notice, setNotice] = useState("");
  const [query, setQuery] = useState("");
  const [kind, setKind] = useState("all");
  const [active, setActive] = useState<OfficeReviewItem | null>(null);
  const [detail, setDetail] = useState<OfficeDetail | null>(null);
  const [detailError, setDetailError] = useState("");
  const [timeline, setTimeline] = useState<TimelineEvent[] | null>(null);
  const [timelineError, setTimelineError] = useState("");
  const [code, setCode] = useState("");
  const [opinion, setOpinion] = useState("");
  const [formError, setFormError] = useState("");
  const [experts, setExperts] = useState<ExpertOption[]>([]);
  const [expertQuery, setExpertQuery] = useState("");
  const [picked, setPicked] = useState<string[]>([]);
  const [pendingAction, setPendingAction] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);

  useEffect(() => {
    let cancelled = false;
    async function load() {
      try {
        const entries = await Promise.all(
          sections.map(
            async (item) => [item.id, await fetchOfficeQueue(item.id)] as const,
          ),
        );
        if (cancelled) return;
        setQueues(Object.fromEntries(entries));
        setLoadError("");
      } catch (error) {
        if (!cancelled)
          setLoadError(
            error instanceof ApiError ? error.message : "审核列表加载失败",
          );
      } finally {
        if (!cancelled) setLoading(false);
      }
    }
    void load();
    return () => {
      cancelled = true;
    };
  }, [version]);

  const rows = useMemo(() => {
    const source = queues[section] ?? [];
    const needle = query.trim();
    return source.filter((item) => {
      const classOk = kind === "all" || item.project_class === kind;
      const text = `${item.title ?? ""}${item.principal_name ?? ""}${item.department_name}${item.code ?? ""}`;
      return classOk && (needle === "" || text.includes(needle));
    });
  }, [kind, query, queues, section]);

  function reload() {
    setLoading(true);
    setVersion((current) => current + 1);
  }

  function closeSheet() {
    if (busy) return;
    setActive(null);
    setDetail(null);
    setDetailError("");
    setTimeline(null);
    setTimelineError("");
    setFormError("");
    setPendingAction(null);
    setPicked([]);
  }

  async function openItem(item: OfficeReviewItem) {
    setActive(item);
    setDetail(null);
    setDetailError("");
    setTimeline(null);
    setTimelineError("");
    setFormError("");
    setCode("");
    setOpinion("");
    setPicked([]);
    setExpertQuery("");
    try {
      const [next, people] = await Promise.all([
        fetchOfficeReview(item.id),
        fetchExperts(""),
      ]);
      setDetail(next);
      setExperts(people);
      try {
        setTimeline(await fetchTimeline(item.id));
      } catch (error) {
        setTimelineError(
          error instanceof ApiError ? error.message : "审核记录读取失败",
        );
      }
    } catch (error) {
      setDetailError(
        error instanceof ApiError ? error.message : "申报书读取失败",
      );
    }
  }

  function ask(action: string) {
    if (action === "approved" && code.trim() === "") {
      setFormError("通过时必须填写封面编号");
      return;
    }
    if (
      (action === "returned" ||
        action === "rejected" ||
        action === "established" ||
        action === "expert_rejected") &&
      opinion.trim() === ""
    ) {
      setFormError(
        action === "established" || action === "expert_rejected"
          ? "请填写结论说明"
          : "退回或不通过必须填写意见",
      );
      return;
    }
    if (action === "assign" && picked.length === 0) {
      setFormError("请选择至少一位专家");
      return;
    }
    setFormError("");
    setPendingAction(action);
  }

  async function confirm() {
    if (!active || !pendingAction) return;
    setBusy(true);
    setFormError("");
    try {
      if (
        pendingAction === "approved" ||
        pendingAction === "returned" ||
        pendingAction === "rejected"
      ) {
        await submitOfficeReview(active.id, {
          decision: pendingAction,
          opinion: opinion.trim(),
          code: code.trim(),
        });
        setNotice(
          pendingAction === "approved"
            ? "已通过，可以指派专家。"
            : pendingAction === "returned"
              ? "已退回给申报人。"
              : "已标记为不通过。",
        );
        setSection(pendingAction === "approved" ? "assign" : pendingAction);
      } else if (pendingAction === "assign") {
        const next = await assignExperts(active.id, picked);
        setDetail(next);
        setPicked([]);
        setNotice("已指派专家，申报进入盲审。");
        setSection("reviewing");
      } else if (
        pendingAction === "established" ||
        pendingAction === "expert_rejected"
      ) {
        await concludeApplication(active.id, {
          decision: pendingAction,
          opinion: opinion.trim(),
        });
        setNotice(
          pendingAction === "established"
            ? "已记为拟立项。结论说明申报人可见。"
            : "已不予立项。结论说明申报人可见。",
        );
        setSection(pendingAction);
        setActive(null);
        setDetail(null);
      }
      if (pendingAction !== "assign") {
        setActive(null);
        setDetail(null);
      }
      setPendingAction(null);
      reload();
    } catch (error) {
      setPendingAction(null);
      setFormError(error instanceof ApiError ? error.message : "操作失败");
    } finally {
      setBusy(false);
    }
  }

  async function removeAssignment(assignmentId: string) {
    if (!active) return;
    setBusy(true);
    setFormError("");
    try {
      const next = await revokeExpert(active.id, assignmentId);
      setDetail(next);
      setNotice(
        next.application.status === "office_approved"
          ? "已撤销全部未评审指派，申报回到待指派。"
          : "已撤销未评审的指派。",
      );
      reload();
    } catch (error) {
      setFormError(error instanceof ApiError ? error.message : "撤销失败");
    } finally {
      setBusy(false);
    }
  }

  const status = detail?.application.status;
  const canReview = status === "dept_approved";
  const canAssign =
    status === "office_approved" || status === "expert_reviewing";
  const visibleExperts = experts.filter((person) => {
    const needle = expertQuery.trim();
    return (
      needle === "" ||
      `${person.real_name}${person.research_expertise ?? ""}`.includes(needle)
    );
  });

  return (
    <WorkspaceShell
      role="office"
      section="review"
      onSection={() => setSection("pending")}
      subtitle="科研处复核"
      sidebar={
        <SideNav
          title="科研处复核"
          items={sections.map((item) => ({
            ...item,
            badge:
              item.id === "pending" || item.id === "conclude"
                ? String((queues[item.id] ?? []).length)
                : undefined,
          }))}
          active={section}
          onSelect={(id) => {
            if (sections.some((item) => item.id === id)) {
              setSection(id as OfficeScope);
              setNotice("");
            }
          }}
        />
      }
    >
      <div className="flex flex-wrap items-end justify-between gap-3">
        <div>
          <h1 className="text-[20px] font-semibold text-ink-900">
            {sections.find((item) => item.id === section)?.label}
          </h1>
          <p className="mt-1 text-[12.5px] text-ink-500">
            科研处审核全院申报，通过后填写封面编号并指派专家。
          </p>
        </div>
        <Button variant="outline" onClick={reload} disabled={loading}>
          <Icon name="mdi:refresh" className="text-base" />
          刷新
        </Button>
      </div>
      {notice ? (
        <p className="mt-4 rounded-lg bg-teal-50 px-3 py-2 text-[12.5px] text-teal-800">
          {notice}
        </p>
      ) : null}
      {loadError ? (
        <p className="mt-4 rounded-lg bg-[#fdf6f6] px-3 py-2 text-[12.5px] text-cinnabar">
          {loadError}
        </p>
      ) : null}
      <div className="mt-4 flex flex-wrap gap-2">
        <div className="relative min-w-55 flex-1">
          <Icon
            name="mdi:magnify"
            className="pointer-events-none absolute top-1.5 left-2.5 text-base text-ink-400"
          />
          <Input
            className="pl-8"
            value={query}
            placeholder="搜索项目、负责人、部门或编号"
            onChange={(event) => setQuery(event.target.value)}
          />
        </div>
        <Select value={kind} onValueChange={setKind}>
          <SelectTrigger aria-label="类别筛选" className="w-35">
            <SelectValue />
          </SelectTrigger>
          <SelectContent>
            <SelectItem value="all">全部类别</SelectItem>
            <SelectItem value="social">社会科学</SelectItem>
            <SelectItem value="natural">自然科学</SelectItem>
            <SelectItem value="dual_high">双高专项</SelectItem>
          </SelectContent>
        </Select>
      </div>
      <div className="mt-4 overflow-hidden rounded-xl border border-ink-200 bg-white">
        <StickyTable
          rows={loading ? [] : rows}
          rowKey={(item) => item.id}
          empty={
            loading
              ? "正在读取审核列表…"
              : query.trim() || kind !== "all"
                ? "没有符合筛选条件的申报。"
                : emptyText[section]
          }
          columns={[
            {
              header: "项目名称",
              width: 220,
              cell: (item) => (
                <span className="font-medium text-ink-900">
                  {item.title || "未填写项目名称"}
                </span>
              ),
            },
            {
              header: "类别",
              width: 110,
              cell: (item) =>
                classLabels[item.project_class] ?? item.project_class,
            },
            {
              header: "负责人",
              width: 120,
              cell: (item) => item.principal_name || "—",
            },
            {
              header: "部门",
              width: 140,
              cell: (item) => item.department_name,
            },
            { header: "编号", width: 120, cell: (item) => item.code || "—" },
            {
              header: "盲审",
              width: 140,
              cell: (item) =>
                item.assigned_count === 0
                  ? "—"
                  : `${item.reviewed_count}/${item.assigned_count}${item.average_score ? ` · ${item.average_score}` : ""}`,
            },
            {
              header: "操作",
              width: 88,
              align: "right",
              cell: (item) => (
                <Button
                  variant="subtle"
                  size="sm"
                  onClick={() => void openItem(item)}
                >
                  {canAct(item.status) ? "处理" : "查看"}
                </Button>
              ),
            },
          ]}
        />
      </div>

      <Dialog
        open={active !== null}
        onOpenChange={(next) => {
          if (!next) closeSheet();
        }}
      >
        <DialogContent className="max-w-215">
          <DialogHeader>
            <DialogTitle>{active?.title || "未填写项目名称"}</DialogTitle>
            <DialogDescription>
              {detail
                ? `${classLabels[detail.application.project_class] ?? detail.application.project_class} · ${statusLabels[detail.application.status] ?? detail.application.status}${detail.code ? ` · ${detail.code}` : ""}`
                : "申报书"}
            </DialogDescription>
          </DialogHeader>
          <DialogBody>
            {detailError ? (
              <p className="mb-3 text-[12.5px] text-cinnabar">{detailError}</p>
            ) : null}
            {detail ? (
              <ApplicationReading detail={detail.application} />
            ) : active && !detailError ? (
              <p className="text-[12.5px] text-ink-400">正在读取申报书…</p>
            ) : null}
            {detail ? (
              <section className="mt-5 border-t border-ink-100 pt-4">
                <h3 className="text-[13px] font-medium text-ink-900">
                  审核记录
                </h3>
                <div className="mt-3">
                  {timelineError ? (
                    <p className="text-[12.5px] text-cinnabar">
                      {timelineError}
                    </p>
                  ) : timeline ? (
                    <ReviewTimeline events={timeline} />
                  ) : (
                    <p className="text-[12.5px] text-ink-400">
                      正在读取审核记录…
                    </p>
                  )}
                </div>
              </section>
            ) : null}
            {detail && detail.assignments.length > 0 ? (
              <section className="mt-5 border-t border-ink-100 pt-4">
                <h3 className="text-[13px] font-medium text-ink-900">
                  专家评审
                  {detail.average_score
                    ? ` · 平均分 ${detail.average_score}`
                    : ""}
                </h3>
                <ul className="mt-2 space-y-2">
                  {detail.assignments.map((item) => (
                    <li
                      key={item.id}
                      className="rounded-lg bg-ink-50 px-3 py-2 text-[12.5px] text-ink-700"
                    >
                      <div className="flex items-center justify-between gap-2">
                        <span>
                          {item.expert_name}
                          {item.research_expertise
                            ? ` · ${item.research_expertise}`
                            : ""}
                        </span>
                        {item.reviewed ? (
                          <span>
                            {item.decision === "approved"
                              ? "建议立项"
                              : "不建议立项"}{" "}
                            · {item.score}
                          </span>
                        ) : (
                          <span className="text-ink-400">未提交</span>
                        )}
                      </div>
                      {item.opinion ? (
                        <p className="mt-1 whitespace-pre-wrap">
                          {item.opinion}
                        </p>
                      ) : null}
                      {!item.reviewed && canAssign ? (
                        <Button
                          className="mt-2"
                          variant="destructive"
                          size="sm"
                          disabled={busy}
                          onClick={() => void removeAssignment(item.id)}
                        >
                          撤销指派
                        </Button>
                      ) : null}
                    </li>
                  ))}
                </ul>
              </section>
            ) : null}
            {detail?.conclusion_opinion ? (
              <p className="mt-4 text-[12.5px] text-ink-700">
                结论说明：{detail.conclusion_opinion}
              </p>
            ) : null}
            {canReview ? (
              <div className="mt-5 space-y-3 border-t border-ink-100 pt-4">
                <label className="block text-[12.5px] text-ink-700">
                  封面编号
                  <Input
                    className="mt-1.5"
                    value={code}
                    placeholder="通过时必填，全局唯一"
                    onChange={(event) => setCode(event.target.value)}
                  />
                </label>
                <label className="block text-[12.5px] text-ink-700">
                  审核意见
                  <Textarea
                    className="mt-1.5"
                    value={opinion}
                    placeholder="退回或不通过时必填"
                    onChange={(event) => setOpinion(event.target.value)}
                  />
                </label>
              </div>
            ) : null}
            {canAssign ? (
              <div className="mt-5 border-t border-ink-100 pt-4">
                <h3 className="text-[13px] font-medium text-ink-900">
                  指派专家
                </h3>
                <Input
                  className="mt-2"
                  value={expertQuery}
                  placeholder="按姓名或研究专长筛选"
                  onChange={(event) => setExpertQuery(event.target.value)}
                />
                <div className="mt-2 max-h-40 space-y-2 overflow-y-auto">
                  {visibleExperts.map((person) => (
                    <label
                      key={person.id}
                      className="flex items-start gap-2 text-[12.5px] text-ink-700"
                    >
                      <Checkbox
                        className="mt-0.5"
                        checked={picked.includes(person.id)}
                        onCheckedChange={(value) =>
                          setPicked((current) =>
                            value === true
                              ? [...current, person.id]
                              : current.filter((id) => id !== person.id),
                          )
                        }
                      />
                      <span>
                        {person.real_name} · {person.department_name}
                        {person.research_expertise
                          ? ` · ${person.research_expertise}`
                          : ""}
                      </span>
                    </label>
                  ))}
                </div>
              </div>
            ) : null}
            {detail?.ready_to_conclude ? (
              <label className="mt-4 block text-[12.5px] text-ink-700">
                结论说明
                <Textarea
                  className="mt-1.5"
                  value={opinion}
                  placeholder="申报人可以看到这段说明"
                  onChange={(event) => setOpinion(event.target.value)}
                />
              </label>
            ) : null}
            {formError ? (
              <p className="mt-2 text-[12.5px] text-cinnabar">{formError}</p>
            ) : null}
          </DialogBody>
          {canReview || canAssign || detail?.ready_to_conclude ? (
            <DialogFooter className="justify-between">
              {canReview ? (
                <Button
                  variant="destructive"
                  disabled={busy || !detail}
                  onClick={() => ask("rejected")}
                >
                  不通过
                </Button>
              ) : (
                <span />
              )}
              <div className="flex gap-2">
                {canReview ? (
                  <Button
                    variant="warning"
                    disabled={busy || !detail}
                    onClick={() => ask("returned")}
                  >
                    退回
                  </Button>
                ) : null}
                {canReview ? (
                  <Button
                    disabled={busy || !detail}
                    onClick={() => ask("approved")}
                  >
                    通过
                  </Button>
                ) : null}
                {canAssign ? (
                  <Button
                    disabled={busy || !detail}
                    onClick={() => ask("assign")}
                  >
                    指派所选专家
                  </Button>
                ) : null}
                {detail?.ready_to_conclude ? (
                  <Button
                    variant="destructive"
                    disabled={busy}
                    onClick={() => ask("expert_rejected")}
                  >
                    不予立项
                  </Button>
                ) : null}
                {detail?.ready_to_conclude ? (
                  <Button disabled={busy} onClick={() => ask("established")}>
                    拟立项
                  </Button>
                ) : null}
              </div>
            </DialogFooter>
          ) : null}
        </DialogContent>
      </Dialog>
      <ConfirmDialog
        open={pendingAction !== null}
        title="确认提交"
        description={confirmCopy(pendingAction, active?.title)}
        confirmLabel="确认"
        busy={busy}
        onConfirm={() => void confirm()}
        onClose={() => {
          if (!busy) setPendingAction(null);
        }}
      />
    </WorkspaceShell>
  );
}

function confirmCopy(action: string | null, title: string | null | undefined) {
  const name = title?.trim() || "未填写项目名称";
  if (action === "assign")
    return `确认把「${name}」发给所选专家。未提交评审前可以撤销。`;
  if (action === "established")
    return `确认把「${name}」记为拟立项。院长办公会通过前，这还不是最终立项。申报人可以看到结论说明。`;
  if (action === "expert_rejected")
    return `确认「${name}」不予立项。申报人可以看到结论说明。`;
  return `确认「${name}」的审核结论。提交后不能在这里修改。`;
}

function canAct(status: string) {
  return (
    status === "dept_approved" ||
    status === "office_approved" ||
    status === "expert_reviewing"
  );
}

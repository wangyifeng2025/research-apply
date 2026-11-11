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
import {
  classLabels,
  statusLabels,
  type ApplicationDetail,
} from "@/lib/applications";
import { fetchTimeline, type TimelineEvent } from "@/lib/timeline";
import {
  fetchDepartmentQueue,
  fetchDepartmentReview,
  submitDepartmentReview,
  type DepartmentDecision,
  type DepartmentReviewItem,
} from "@/lib/department-reviews";

type SectionId = "pending" | "approved" | "returned" | "rejected";

const sections: Array<{ id: SectionId; label: string; icon: string }> = [
  { id: "pending", label: "待审核", icon: "mdi:inbox-arrow-down-outline" },
  { id: "approved", label: "已通过", icon: "mdi:check-decagram-outline" },
  { id: "returned", label: "已退回", icon: "mdi:file-undo-outline" },
  { id: "rejected", label: "不通过", icon: "mdi:gavel" },
];

const confirmations = [
  ["facts", "申报内容是否属实"],
  ["team", "负责人及参加者是否适合承担本项目"],
  ["resources", "本部门能否提供完成项目所需的时间和条件"],
  ["manage", "本部门是否同意承担管理任务和信誉保证"],
] as const;

type CheckKey = (typeof confirmations)[number][0];

const emptyChecks: Record<CheckKey, boolean> = {
  facts: false,
  team: false,
  resources: false,
  manage: false,
};

const decisionCopy: Record<
  DepartmentDecision,
  { title: string; confirm: string; done: string }
> = {
  approved: {
    title: "确认通过",
    confirm: "确认通过",
    done: "已通过，申报进入科研处审核。",
  },
  returned: {
    title: "确认退回",
    confirm: "确认退回",
    done: "已退回给申报人。",
  },
  rejected: {
    title: "确认不通过",
    confirm: "确认不通过",
    done: "已标记为不通过。",
  },
};

export function DepartmentWorkspace() {
  const [section, setSection] = useState<SectionId>("pending");
  const [pending, setPending] = useState<DepartmentReviewItem[]>([]);
  const [reviewed, setReviewed] = useState<DepartmentReviewItem[]>([]);
  const [loading, setLoading] = useState(true);
  const [version, setVersion] = useState(0);
  const [loadError, setLoadError] = useState("");
  const [notice, setNotice] = useState("");
  const [query, setQuery] = useState("");
  const [kind, setKind] = useState("all");
  const [active, setActive] = useState<DepartmentReviewItem | null>(null);
  const [detail, setDetail] = useState<ApplicationDetail | null>(null);
  const [detailError, setDetailError] = useState("");
  const [timeline, setTimeline] = useState<TimelineEvent[] | null>(null);
  const [timelineError, setTimelineError] = useState("");
  const [checks, setChecks] = useState(emptyChecks);
  const [opinion, setOpinion] = useState("");
  const [formError, setFormError] = useState("");
  const [pendingDecision, setPendingDecision] =
    useState<DepartmentDecision | null>(null);
  const [busy, setBusy] = useState(false);

  useEffect(() => {
    let cancelled = false;
    async function load() {
      try {
        const [pendingItems, reviewedItems] = await Promise.all([
          fetchDepartmentQueue("pending"),
          fetchDepartmentQueue("reviewed"),
        ]);
        if (cancelled) return;
        setPending(pendingItems);
        setReviewed(reviewedItems);
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
    const source =
      section === "pending"
        ? pending
        : reviewed.filter((item) => item.decision === section);
    const needle = query.trim();
    return source.filter((item) => {
      const classOk = kind === "all" || item.project_class === kind;
      const text = `${item.title ?? ""}${item.principal_name ?? ""}${item.department_name}`;
      return classOk && (needle === "" || text.includes(needle));
    });
  }, [kind, pending, query, reviewed, section]);

  const heading =
    sections.find((item) => item.id === section)?.label ?? "待审核";

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
    setPendingDecision(null);
  }

  async function openItem(item: DepartmentReviewItem) {
    setActive(item);
    setDetail(null);
    setDetailError("");
    setTimeline(null);
    setTimelineError("");
    setFormError("");
    setOpinion("");
    setChecks(emptyChecks);
    try {
      setDetail(await fetchDepartmentReview(item.id));
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

  function ask(decision: DepartmentDecision) {
    if (!active || active.decision) return;
    if (
      decision === "approved" &&
      Object.values(checks).some((checked) => !checked)
    ) {
      setFormError("通过时四项确认都要勾选");
      return;
    }
    if (decision !== "approved" && opinion.trim() === "") {
      setFormError("退回或不通过必须填写意见");
      return;
    }
    setFormError("");
    setPendingDecision(decision);
  }

  async function submit() {
    if (!active || !pendingDecision) return;
    setBusy(true);
    setFormError("");
    try {
      await submitDepartmentReview(active.id, {
        decision: pendingDecision,
        opinion: opinion.trim(),
        facts_accurate: checks.facts,
        team_qualified: checks.team,
        resources_ready: checks.resources,
        agree_to_manage: checks.manage,
      });
      setNotice(decisionCopy[pendingDecision].done);
      setPendingDecision(null);
      setActive(null);
      setDetail(null);
      setSection(pendingDecision);
      reload();
    } catch (error) {
      setPendingDecision(null);
      setFormError(error instanceof ApiError ? error.message : "审核提交失败");
    } finally {
      setBusy(false);
    }
  }

  return (
    <WorkspaceShell
      role="department"
      section="review"
      onSection={() => setSection("pending")}
      subtitle="系部初审"
      sidebar={
        <SideNav
          title="系部初审"
          items={sections.map((item) => ({
            ...item,
            badge: item.id === "pending" ? String(pending.length) : undefined,
          }))}
          active={section}
          onSelect={(id) => {
            if (
              id === "pending" ||
              id === "approved" ||
              id === "returned" ||
              id === "rejected"
            ) {
              setSection(id);
              setNotice("");
            }
          }}
        />
      }
    >
      <div className="flex flex-wrap items-end justify-between gap-3">
        <div>
          <h1 className="text-[20px] font-semibold text-ink-900">{heading}</h1>
          <p className="mt-1 text-[12.5px] text-ink-500">
            {section === "pending"
              ? "只显示本部门已提交、尚未审核的申报书。"
              : "只显示本人已经审过的申报书。"}
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
            placeholder="搜索项目名称、负责人"
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
              : emptyCopy(section, query.trim() !== "" || kind !== "all")
          }
          columns={[
            {
              header: "项目名称",
              width: 240,
              cell: (item) => (
                <span className="font-medium text-ink-900">
                  {item.title || "未填写项目名称"}
                </span>
              ),
            },
            {
              header: "类别",
              width: 120,
              cell: (item) =>
                classLabels[item.project_class] ?? item.project_class,
            },
            {
              header: "负责人",
              width: 140,
              cell: (item) => (
                <div>
                  <p>{item.principal_name || "—"}</p>
                  <p className="mt-0.5 text-[11px] text-ink-400">
                    {item.professional_title || "未填职称"}
                  </p>
                </div>
              ),
            },
            {
              header: "部门",
              width: 140,
              cell: (item) => item.department_name,
            },
            {
              header: "经费（万元）",
              width: 120,
              cell: (item) => (
                <span className="tabular-nums">{item.budget_total}</span>
              ),
            },
            {
              header: "提交时间",
              width: 140,
              cell: (item) =>
                formatTime(
                  section === "pending" ? item.submitted_at : item.reviewed_at,
                ),
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
                  {item.decision ? "查看" : "审核"}
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
              {active
                ? `${classLabels[active.project_class] ?? active.project_class} · ${statusLabels[active.status] ?? active.status}`
                : "申报书"}
            </DialogDescription>
          </DialogHeader>
          <DialogBody>
            {detailError ? (
              <p className="mb-3 text-[12.5px] text-cinnabar">{detailError}</p>
            ) : null}
            {detail ? (
              <ApplicationReading detail={detail} />
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
            {active ? (
              <fieldset
                className="mt-5 border-t border-ink-100 pt-4"
                disabled={Boolean(active.decision) || busy}
              >
                <legend className="text-[13px] font-medium text-ink-900">
                  所在部门意见
                </legend>
                <div className="mt-3 space-y-2.5">
                  {confirmations.map(([key, label]) => (
                    <label
                      key={key}
                      className="flex items-start gap-2 text-[12.5px] leading-5 text-ink-700"
                    >
                      <Checkbox
                        className="mt-0.5"
                        disabled={Boolean(active.decision) || busy}
                        checked={
                          active.decision
                            ? recordedCheck(active, key)
                            : checks[key]
                        }
                        onCheckedChange={(value) =>
                          setChecks((current) => ({
                            ...current,
                            [key]: value === true,
                          }))
                        }
                      />
                      {label}
                    </label>
                  ))}
                </div>
                <label className="mt-4 block text-[12.5px] text-ink-700">
                  审核意见
                  <Textarea
                    className="mt-1.5"
                    value={active.decision ? (active.opinion ?? "") : opinion}
                    placeholder="退回或不通过时必填"
                    onChange={(event) => setOpinion(event.target.value)}
                  />
                </label>
                {formError ? (
                  <p className="mt-2 text-[12.5px] text-cinnabar">
                    {formError}
                  </p>
                ) : null}
              </fieldset>
            ) : null}
          </DialogBody>
          {active && !active.decision ? (
            <DialogFooter className="justify-between">
              <Button
                variant="destructive"
                disabled={busy || !detail}
                onClick={() => ask("rejected")}
              >
                不通过
              </Button>
              <div className="flex gap-2">
                <Button
                  variant="warning"
                  disabled={busy || !detail}
                  onClick={() => ask("returned")}
                >
                  退回
                </Button>
                <Button
                  disabled={busy || !detail}
                  onClick={() => ask("approved")}
                >
                  通过
                </Button>
              </div>
            </DialogFooter>
          ) : null}
        </DialogContent>
      </Dialog>

      <ConfirmDialog
        open={pendingDecision !== null}
        title={
          pendingDecision ? decisionCopy[pendingDecision].title : "确认审核"
        }
        description={
          active
            ? `「${active.title || "未填写项目名称"}」提交后不能在这里改结论。`
            : ""
        }
        confirmLabel={
          pendingDecision ? decisionCopy[pendingDecision].confirm : "确认"
        }
        busy={busy}
        onConfirm={() => void submit()}
        onClose={() => {
          if (!busy) setPendingDecision(null);
        }}
      />
    </WorkspaceShell>
  );
}

function recordedCheck(item: DepartmentReviewItem, key: CheckKey) {
  if (key === "facts") return item.facts_accurate === true;
  if (key === "team") return item.team_qualified === true;
  if (key === "resources") return item.resources_ready === true;
  return item.agree_to_manage === true;
}

function emptyCopy(section: SectionId, filtered: boolean) {
  if (filtered) return "没有符合筛选条件的申报。";
  if (section === "pending")
    return "本部门还没有待审申报。教师提交后会出现在这里。";
  if (section === "approved") return "还没有通过的申报。";
  if (section === "returned") return "还没有退回的申报。";
  return "还没有不通过的申报。";
}

function formatTime(value: string | null) {
  if (!value) return "—";
  const date = new Date(value);
  if (Number.isNaN(date.getTime())) return value;
  return new Intl.DateTimeFormat("zh-CN", {
    month: "2-digit",
    day: "2-digit",
    hour: "2-digit",
    minute: "2-digit",
  }).format(date);
}

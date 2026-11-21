"use client";

import { useEffect, useState } from "react";

import { AttachmentPanel } from "@/components/attachment-panel";
import { Button } from "@/components/ui/button";
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
import { StickyTable } from "@/components/ui/sticky-table";
import { Textarea } from "@/components/ui/textarea";
import { SideNav, WorkspaceShell } from "@/components/workspace-shell";
import { ApiError } from "@/lib/api";
import { classLabels } from "@/lib/applications";
import {
  fetchExpertQueue,
  fetchExpertReview,
  submitExpertReview,
  type BlindApplication,
  type ExpertDetail,
  type ExpertReviewItem,
} from "@/lib/expert-reviews";

export function ExpertWorkspace() {
  const [section, setSection] = useState<"pending" | "reviewed">("pending");
  const [pending, setPending] = useState<ExpertReviewItem[]>([]);
  const [reviewed, setReviewed] = useState<ExpertReviewItem[]>([]);
  const [loading, setLoading] = useState(true);
  const [version, setVersion] = useState(0);
  const [loadError, setLoadError] = useState("");
  const [notice, setNotice] = useState("");
  const [active, setActive] = useState<ExpertReviewItem | null>(null);
  const [detail, setDetail] = useState<ExpertDetail | null>(null);
  const [detailError, setDetailError] = useState("");
  const [score, setScore] = useState("");
  const [opinion, setOpinion] = useState("");
  const [formError, setFormError] = useState("");
  const [pendingDecision, setPendingDecision] = useState<
    "approved" | "rejected" | null
  >(null);
  const [busy, setBusy] = useState(false);

  useEffect(() => {
    let cancelled = false;
    async function load() {
      try {
        const [pendingItems, reviewedItems] = await Promise.all([
          fetchExpertQueue("pending"),
          fetchExpertQueue("reviewed"),
        ]);
        if (cancelled) return;
        setPending(pendingItems);
        setReviewed(reviewedItems);
        setLoadError("");
      } catch (error) {
        if (!cancelled)
          setLoadError(
            error instanceof ApiError ? error.message : "评审列表加载失败",
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

  const rows = section === "pending" ? pending : reviewed;

  async function openItem(item: ExpertReviewItem) {
    setActive(item);
    setDetail(null);
    setDetailError("");
    setFormError("");
    setScore("");
    setOpinion("");
    try {
      setDetail(await fetchExpertReview(item.id));
    } catch (error) {
      setDetailError(
        error instanceof ApiError ? error.message : "申报书读取失败",
      );
    }
  }

  function ask(decision: "approved" | "rejected") {
    if (score.trim() === "") {
      setFormError("请填写 0 到 100 的评分");
      return;
    }
    if (opinion.trim() === "") {
      setFormError("请填写评审意见");
      return;
    }
    setFormError("");
    setPendingDecision(decision);
  }

  async function confirm() {
    if (!active || !pendingDecision) return;
    setBusy(true);
    try {
      await submitExpertReview(active.id, {
        decision: pendingDecision,
        opinion: opinion.trim(),
        score: score.trim(),
      });
      setNotice(
        pendingDecision === "approved"
          ? "已提交建议立项。"
          : "已提交不建议立项。",
      );
      setSection("reviewed");
      setActive(null);
      setDetail(null);
      setPendingDecision(null);
      setLoading(true);
      setVersion((current) => current + 1);
    } catch (error) {
      setPendingDecision(null);
      setFormError(error instanceof ApiError ? error.message : "评审提交失败");
    } finally {
      setBusy(false);
    }
  }

  return (
    <WorkspaceShell
      role="expert"
      section="review"
      onSection={() => setSection("pending")}
      subtitle="专家盲审"
      sidebar={
        <SideNav
          title="专家盲审"
          items={[
            {
              id: "pending",
              label: "待评",
              icon: "mdi:inbox-arrow-down-outline",
              badge: String(pending.length),
            },
            {
              id: "reviewed",
              label: "已评",
              icon: "mdi:check-decagram-outline",
            },
          ]}
          active={section}
          onSelect={(id) => {
            if (id === "pending" || id === "reviewed") setSection(id);
          }}
        />
      }
    >
      <h1 className="text-[20px] font-semibold text-ink-900">
        {section === "pending" ? "待评项目" : "已评项目"}
      </h1>
      <p className="mt-1 max-w-2xl text-[12.5px] leading-5 text-ink-500">
        这里只显示项目类别和名称。评审页不包含负责人、部门和参加者。
      </p>
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
      <div className="mt-4 overflow-hidden rounded-xl border border-ink-200 bg-white">
        <StickyTable
          rows={loading ? [] : rows}
          rowKey={(item) => item.id}
          empty={loading ? "正在读取评审列表…" : section === "pending" ? "当前没有指派给本人的待评申报。" : "还没有提交过评审。"}
          columns={[
            { header: "项目名称", width: 320, cell: (item) => <span className="font-medium text-ink-900">{item.title || "未填写项目名称"}</span> },
            { header: "类别", width: 140, cell: (item) => classLabels[item.project_class] ?? item.project_class },
            { header: "评分", width: 100, cell: (item) => (item.score ? `${item.score} 分` : "—") },
            {
              header: "操作",
              width: 88,
              align: "right",
              cell: (item) => (
                <Button variant="subtle" size="sm" onClick={() => void openItem(item)}>
                  {item.decision ? "查看" : "评审"}
                </Button>
              ),
            },
          ]}
        />
      </div>
      <Dialog
        open={active !== null}
        onOpenChange={(next) => {
          if (!next && !busy) {
            setActive(null);
            setDetail(null);
          }
        }}
      >
        <DialogContent className="max-w-215">
          <DialogHeader>
            <DialogTitle>
              {detail?.application.title || active?.title || "未填写项目名称"}
            </DialogTitle>
            <DialogDescription>
              {classLabels[
                detail?.application.project_class ?? active?.project_class ?? ""
              ] ?? "申报内容"}
            </DialogDescription>
          </DialogHeader>
          <DialogBody>
            {detailError ? (
              <p className="text-[12.5px] text-cinnabar">{detailError}</p>
            ) : null}
            {detail ? (
              <BlindReading application={detail.application} />
            ) : active && !detailError ? (
              <p className="text-[12.5px] text-ink-400">正在读取申报内容…</p>
            ) : null}
            {detail?.review ? (
              <p className="mt-4 text-[12.5px] leading-6 text-ink-700">
                已提交：
                {detail.review.decision === "approved"
                  ? "建议立项"
                  : "不建议立项"}
                ，{detail.review.score} 分。{detail.review.opinion}
              </p>
            ) : detail ? (
              <div className="mt-5 space-y-3 border-t border-ink-100 pt-4">
                <label className="block text-[12.5px] text-ink-700">
                  评分（0–100）
                  <Input
                    className="mt-1.5"
                    value={score}
                    inputMode="decimal"
                    placeholder="例如 86.5"
                    onChange={(event) => setScore(clampScore(event.target.value))}
                  />
                </label>
                <label className="block text-[12.5px] text-ink-700">
                  评审意见
                  <Textarea className="mt-1.5" value={opinion} onChange={(event) => setOpinion(event.target.value)} />
                </label>
                {formError ? (
                  <p className="text-[12.5px] text-cinnabar">{formError}</p>
                ) : null}
              </div>
            ) : null}
          </DialogBody>
          {detail && !detail.review ? (
            <DialogFooter>
              <Button
                variant="destructive"
                disabled={busy}
                onClick={() => ask("rejected")}
              >
                不建议立项
              </Button>
              <Button disabled={busy} onClick={() => ask("approved")}>
                建议立项
              </Button>
            </DialogFooter>
          ) : null}
        </DialogContent>
      </Dialog>
      <ConfirmDialog
        open={pendingDecision !== null}
        title="确认提交评审"
        description="提交后不能修改评分和意见。"
        confirmLabel="确认提交"
        busy={busy}
        onConfirm={() => void confirm()}
        onClose={() => {
          if (!busy) setPendingDecision(null);
        }}
      />
    </WorkspaceShell>
  );
}

function clampScore(raw: string) {
  if (raw.trim() === "") return "";
  const normalized = raw.match(/^\d*\.?\d{0,2}/)?.[0] ?? "";
  if (normalized === "" || normalized === ".") return "";
  if (Number(normalized) > 100) return "100";
  return normalized;
}

function BlindReading({ application }: { application: BlindApplication }) {
  const natural = application.project_class !== "social";
  return (
    <div className="space-y-4 text-[12.5px] leading-6 text-ink-700">
      <Paragraph title="关键词" text={application.keywords} />
      <section>
        <h3 className="font-medium text-ink-900">
          经费合计 {application.budget.total} 万元
        </h3>
        <p className="text-ink-500">
          直接费用 {application.budget.direct_total}，间接费用{" "}
          {application.budget.indirect_total}
        </p>
      </section>
      {natural ? (
        <>
          <Paragraph title="项目简介" text={application.ns_profile} />
          <Paragraph title="主要研究内容" text={application.ns_content} />
          <Paragraph
            title="社会效益说明"
            text={
              typeof application.ns_indicators.benefit_note === "string"
                ? application.ns_indicators.benefit_note
                : null
            }
          />
          <List
            title="计划进度"
            rows={application.ns_schedules.map((item) =>
              [item.start_date, item.end_date, item.goal]
                .filter(Boolean)
                .join(" "),
            )}
          />
          <List
            title="仪器设备"
            rows={application.ns_equipments.map(
              (item) => `${item.name_spec} × ${item.quantity}`,
            )}
          />
        </>
      ) : (
        <>
          <Paragraph
            title="选题依据"
            text={application.ss_design.topic_basis}
          />
          <Paragraph
            title="研究内容"
            text={application.ss_design.research_content}
          />
          <Paragraph
            title="研究方法"
            text={application.ss_design.methodology}
          />
          <Paragraph title="创新之处" text={application.ss_design.innovation} />
          <Paragraph
            title="预期成果"
            text={application.ss_design.expected_outcome}
          />
          <Paragraph
            title="研究基础"
            text={application.ss_foundation.research_basis}
          />
          <Paragraph
            title="保障条件"
            text={application.ss_foundation.support_conditions}
          />
          <List
            title="前期成果"
            rows={application.ss_achievements.map((item) => item.title)}
          />
          <List
            title="阶段成果"
            rows={application.ss_stage_outputs.map((item) => item.title)}
          />
          <List
            title="最终成果"
            rows={application.ss_final_outputs.map((item) => item.title)}
          />
        </>
      )}
      <AttachmentPanel applicationId={application.id} items={application.attachments} editable={false} />
    </div>
  );
}

function Paragraph({
  title,
  text,
}: {
  title: string;
  text: string | null | undefined;
}) {
  if (!text?.trim()) return null;
  return (
    <section>
      <h3 className="font-medium text-ink-900">{title}</h3>
      <p className="mt-1 whitespace-pre-wrap">{text}</p>
    </section>
  );
}

function List({ title, rows }: { title: string; rows: string[] }) {
  const filled = rows.map((row) => row.trim()).filter(Boolean);
  if (filled.length === 0) return null;
  return (
    <section>
      <h3 className="font-medium text-ink-900">{title}</h3>
      <ul className="mt-1 list-disc pl-4">
        {filled.map((row) => (
          <li key={row}>{row}</li>
        ))}
      </ul>
    </section>
  );
}

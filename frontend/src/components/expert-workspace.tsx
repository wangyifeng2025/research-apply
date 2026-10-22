"use client";

import { WorkspaceShell } from "@/components/workspace-shell";

const assignments = [
  { code: "NS-2026-014", title: "工业机器人视觉伺服控制方法研究", kind: "自然科学", due: "10-24" },
  { code: "SS-2026-031", title: "地方高校教师教学能力发展研究", kind: "社会科学", due: "10-26" },
];

export function ExpertWorkspace() {
  return (
    <WorkspaceShell role="expert" section="review" onSection={() => undefined} subtitle="专家盲审 · 身份隔离">
      <p className="text-[11px] text-ink-400">专家盲审</p>
      <h1 className="mt-1.5 text-[22px] font-semibold tracking-tight text-ink-900">待评项目</h1>
      <p className="mt-1 max-w-2xl text-[12px] leading-5 text-ink-500">
        专家只看申报内容。申请人、系部和公示页都不能看到评审专家身份。
      </p>
      <div className="mt-4 grid gap-3">
        {assignments.map((item) => (
          <article key={item.code} className="rounded-xl border border-ink-200/80 bg-white p-4">
            <div className="flex flex-wrap items-center justify-between gap-2">
              <h2 className="text-[14px] font-semibold text-ink-900">{item.title}</h2>
              <span className="rounded-full bg-gold-50 px-2 py-0.5 text-[10.5px] text-gold-800">待评分</span>
            </div>
            <p className="mt-1 text-[12px] text-ink-500">
              {item.code} · {item.kind} · 请于 {item.due} 前提交
            </p>
          </article>
        ))}
      </div>
    </WorkspaceShell>
  );
}

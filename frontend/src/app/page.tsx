import Link from "next/link";

import { cycleStatusLabel, fetchHealth, fetchMeta } from "@/lib/api";

export const dynamic = "force-dynamic";

export default async function HomePage() {
  const [health, meta] = await Promise.all([fetchHealth(), fetchMeta()]);
  const connected = health?.database === "up";

  return (
    <div className="grid gap-10">
      <section className="rise grid items-end gap-8 lg:grid-cols-[1.4fr_0.6fr]">
        <div>
          <p className="text-sm tracking-[0.35em] text-[var(--color-cinnabar)]">二〇二六</p>
          <h1 className="mt-3 max-w-3xl font-serif text-5xl leading-tight sm:text-6xl">
            把申报书做成可审核的卷宗
          </h1>
          <p className="mt-5 max-w-2xl text-base leading-8 text-[var(--color-ink-soft)]">
            教师填写社会科学或自然科学申报，经系审核、科研处审核后进入专家盲审。封面编号由科研处分配，关键词不超过五个。
          </p>
        </div>
        <aside className="flex items-center justify-between gap-4 border border-[var(--color-line)] bg-[var(--color-card)] p-5">
          <div>
            <p className="text-xs tracking-[0.2em] text-[var(--color-ink-soft)]">本批次</p>
            <p className="mt-2 font-serif text-2xl">{meta?.cycle?.title ?? "2026年院级科研项目申报"}</p>
            <p className="mt-2 text-sm text-[var(--color-ink-soft)]">
              {meta?.cycle ? cycleStatusLabel(meta.cycle.status) : "等待数据库"}
              {meta ? ` · 经费科目 ${meta.budget_subject_count} 项` : ""}
            </p>
          </div>
          <div className="seal">科研处</div>
        </aside>
      </section>

      <section className="grid gap-4 md:grid-cols-2">
        <TrackCard
          href="/applications/social/new"
          eyebrow="附件 2"
          title="社会科学类"
          label="B.社会科学类"
          tone="text-[var(--color-indigo)]"
          points={["设计论证与研究基础", "阶段性成果最多 10 项", "图书资料、数据采集等直接费用"]}
        />
        <TrackCard
          href="/applications/natural/new"
          eyebrow="附件 3"
          title="自然科学类"
          label="A.自然科学类 / D.双高专项"
          tone="text-[var(--color-pine)]"
          points={["量化技术经济指标", "计划进度与仪器设备", "设备费按购置、试制、改造分列"]}
        />
      </section>

      <section>
        <h2 className="font-serif text-2xl">审核顺序</h2>
        <ol className="mt-4 grid gap-3 sm:grid-cols-4">
          {(meta?.workflow ?? fallbackWorkflow).map((step, index) => (
            <li key={step.code} className="border-t-2 border-[var(--color-ink)] pt-3">
              <p className="text-xs text-[var(--color-ink-soft)]">0{index + 1}</p>
              <p className="mt-1 font-serif text-xl">{step.label}</p>
            </li>
          ))}
        </ol>
      </section>

      <p className="text-sm text-[var(--color-ink-soft)]">
        数据库 {meta?.database ?? "research_apply_db"}{" "}
        {connected ? "已连接" : "未连接，请先启动后端"}
      </p>
    </div>
  );
}

const fallbackWorkflow = [
  { code: "teacher", label: "教师填报" },
  { code: "department", label: "系审核" },
  { code: "research_office", label: "科研处审核" },
  { code: "expert", label: "专家盲审" },
];

function TrackCard({
  href,
  eyebrow,
  title,
  label,
  tone,
  points,
}: {
  href: string;
  eyebrow: string;
  title: string;
  label: string;
  tone: string;
  points: string[];
}) {
  return (
    <Link
      href={href}
      className="rise block border border-[var(--color-line)] bg-[var(--color-card)] p-6 transition hover:-translate-y-0.5"
    >
      <p className={`text-xs tracking-[0.22em] ${tone}`}>{eyebrow}</p>
      <h2 className="mt-3 font-serif text-3xl">{title}</h2>
      <p className="mt-1 text-sm text-[var(--color-ink-soft)]">{label}</p>
      <ul className="mt-5 grid gap-2 text-sm leading-6">
        {points.map((point) => (
          <li key={point}>{point}</li>
        ))}
      </ul>
    </Link>
  );
}

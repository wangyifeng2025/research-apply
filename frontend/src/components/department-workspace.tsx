"use client";

import { useMemo, useState } from "react";

import { Icon } from "@/components/icon";
import { SideNav, WorkspaceShell } from "@/components/workspace-shell";

type Status = "pending" | "passed" | "returned";

type Item = {
  code: string;
  title: string;
  kind: "社会科学" | "自然科学";
  leader: string;
  rank: string;
  dept: string;
  budget: string;
  time: string;
  note: string;
  status: Status;
};

const seed: Item[] = [
  { code: "SH2026-0192", title: "数字普惠金融促进县域共同富裕的机制与路径研究", kind: "社会科学", leader: "周晓彤", rank: "副教授", dept: "公共管理系", budget: "12.0", time: "10-20 08:52", note: "3 个附件", status: "pending" },
  { code: "ZR2026-0051", title: "面向锂电池极片缺陷的多光谱视觉检测方法研究", kind: "自然科学", leader: "陈博", rank: "讲师", dept: "智能制造学院", budget: "18.0", time: "10-19 17:20", note: "4 个附件", status: "pending" },
  { code: "SH2026-0188", title: "高校教师数字素养评价指标体系构建与应用研究", kind: "社会科学", leader: "林雅琴", rank: "教授", dept: "教育学系", budget: "9.5", time: "10-19 11:05", note: "2 个附件", status: "pending" },
  { code: "ZR2026-0049", title: "基于数字孪生的智能制造车间调度优化研究", kind: "自然科学", leader: "孙浩", rank: "副教授", dept: "智能制造学院", budget: "20.0", time: "10-18 15:40", note: "5 个附件", status: "pending" },
  { code: "SH2026-0181", title: "乡村振兴背景下农村养老服务多元供给机制研究", kind: "社会科学", leader: "何雨欣", rank: "讲师", dept: "社会学系", budget: "10.0", time: "10-18 09:31", note: "补正后重提", status: "pending" },
];

export function DepartmentWorkspace() {
  const [section, setSection] = useState("pending");
  const [items, setItems] = useState(seed);
  const [query, setQuery] = useState("");
  const [kind, setKind] = useState("全部类别");
  const [picked, setPicked] = useState<string[]>([]);
  const [active, setActive] = useState<Item | null>(null);
  const [opinion, setOpinion] = useState("经审核，申报内容基本属实，同意推荐上报科研处。");
  const [toast, setToast] = useState("");

  const nav = [
    { id: "queue", label: "审核工作台", icon: "mdi:clipboard-text-search-outline" },
    { id: "pending", label: "待审核", icon: "mdi:inbox-arrow-down-outline", badge: String(items.filter((item) => item.status === "pending").length) },
    { id: "passed", label: "已通过", icon: "mdi:check-decagram-outline" },
    { id: "returned", label: "已退回", icon: "mdi:file-undo-outline" },
    { id: "notice", label: "立项公示", icon: "mdi:bullhorn-outline" },
    { id: "stats", label: "审核统计", icon: "mdi:chart-box-outline" },
  ];

  const status: Status | "all" = section === "passed" || section === "returned" || section === "pending" ? section : "pending";
  const rows = useMemo(
    () => items.filter((item) => (section === "queue" || section === "stats" || section === "notice" ? true : item.status === status) && (kind === "全部类别" || item.kind === kind) && `${item.title}${item.code}${item.leader}`.includes(query.trim())),
    [items, kind, query, section, status],
  );

  function notify(message: string) {
    setToast(message);
    window.setTimeout(() => setToast(""), 2200);
  }

  function decide(codes: string[], next: Status) {
    setItems((current) => current.map((item) => (codes.includes(item.code) ? { ...item, status: next } : item)));
    setPicked([]);
    setActive(null);
    notify(next === "passed" ? "已通过并留在本页" : "已退回给申报人");
  }

  return (
    <WorkspaceShell role="department" section="review" onSection={() => setSection("pending")} subtitle="系部初审 · 经济管理学院" sidebar={<SideNav title="系部初审" items={nav} active={section} onSelect={setSection} />}>
      <div className="grid grid-cols-2 gap-3 xl:grid-cols-4">
        <Mini label="待审核" value={String(items.filter((item) => item.status === "pending").length)} icon="mdi:inbox-arrow-down-outline" />
        <Mini label="今日已审" value={String(items.filter((item) => item.status !== "pending").length)} icon="mdi:check-decagram-outline" />
        <Mini label="本批次退回" value={String(items.filter((item) => item.status === "returned").length)} icon="mdi:file-undo-outline" />
        <Mini label="平均审核时长" value="1.6 天" icon="mdi:speedometer" />
      </div>

      {section === "notice" ? (
        <article className="mt-4 rounded-xl bg-ink-900 p-5 text-white">
          <p className="text-[12px] text-teal-200">公示公告</p>
          <h1 className="mt-2 text-[20px] font-semibold">关于 2026 年度第一批拟立项名单的公示</h1>
          <p className="mt-2 text-[12px] text-ink-200">公示期 2026-11-05 至 2026-11-18 · 本系拟立项 6 项</p>
        </article>
      ) : null}

      {section === "stats" ? (
        <section className="mt-4 rounded-xl border border-ink-200/80 bg-white p-4 text-[13px] text-ink-600">
          本批次收到 28 项，已通过 16 项，退回 7 项，仍有 {items.filter((item) => item.status === "pending").length} 项待审。通过率按已办结项目计算。
        </section>
      ) : null}

      {section !== "notice" && section !== "stats" ? (
        <section className="mt-4 rounded-xl border border-ink-200/80 bg-white shadow-sm">
          <div className="flex flex-wrap items-center gap-2 border-b border-ink-100 px-4 py-3">
            <label className="flex h-8 items-center gap-1 rounded-lg border border-ink-200 px-2 text-ink-400">
              <Icon name="mdi:magnify" />
              <input className="w-44 bg-transparent text-[12px] text-ink-800 outline-none" placeholder="项目、编号、负责人" value={query} onChange={(event) => setQuery(event.target.value)} />
            </label>
            <select className="field-input w-auto" value={kind} onChange={(event) => setKind(event.target.value)}>
              <option>全部类别</option>
              <option>社会科学</option>
              <option>自然科学</option>
            </select>
            <span className="text-[11px] text-ink-400">已选 {picked.length} 项</span>
            <div className="ml-auto flex gap-1.5">
              <button type="button" onClick={() => decide(picked, "passed")} className="inline-flex h-7 items-center gap-1 rounded-md border border-teal-200 bg-teal-50 px-2.5 text-[10.5px] text-teal-700"><Icon name="mdi:check-all" />批量通过</button>
              <button type="button" onClick={() => decide(picked, "returned")} className="inline-flex h-7 items-center gap-1 rounded-md border border-gold-200 bg-gold-50 px-2.5 text-[10.5px] text-gold-700"><Icon name="mdi:undo-variant" />批量退回</button>
            </div>
          </div>
          <div className="overflow-x-auto">
            <table className="w-full min-w-[900px] text-left text-[12px]">
              <thead>
                <tr className="bg-ink-50/70 text-[10.5px] text-ink-500">
                  <th className="w-10 px-4 py-2.5"><input type="checkbox" aria-label="全选" checked={rows.length > 0 && rows.every((item) => picked.includes(item.code))} onChange={(event) => setPicked(event.target.checked ? rows.map((item) => item.code) : [])} /></th>
                  {["项目名称 / 编号", "类别", "负责人", "申报经费", "提交时间", "状态", "操作"].map((head) => <th key={head} className="px-3 py-2.5">{head}</th>)}
                </tr>
              </thead>
              <tbody className="divide-y divide-ink-100">
                {rows.map((item) => (
                  <tr key={item.code} className="hover:bg-ink-50/40">
                    <td className="px-4 py-3"><input type="checkbox" aria-label={`选择 ${item.code}`} checked={picked.includes(item.code)} onChange={() => setPicked((current) => current.includes(item.code) ? current.filter((id) => id !== item.code) : [...current, item.code])} /></td>
                    <td className="px-3 py-3"><p className="font-medium text-ink-800">{item.title}</p><p className="mt-0.5 text-[10px] text-ink-400">{item.code} · {item.dept} · {item.note}</p></td>
                    <td className="px-3 py-3"><span className={`rounded border px-1.5 py-0.5 text-[10px] ${item.kind === "社会科学" ? "border-gold-200 bg-gold-50 text-gold-700" : "border-teal-200 bg-teal-50 text-teal-700"}`}>{item.kind}</span></td>
                    <td className="px-3 py-3"><p>{item.leader}</p><p className="text-[10px] text-ink-400">{item.rank}</p></td>
                    <td className="px-3 py-3">{item.budget} 万元</td>
                    <td className="px-3 py-3 text-ink-500">{item.time}</td>
                    <td className="px-3 py-3"><StatusBadge status={item.status} /></td>
                    <td className="px-3 py-3 text-right"><button type="button" onClick={() => setActive(item)} className="rounded-md bg-teal-700 px-2.5 py-1 text-[10.5px] text-white">审核</button></td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        </section>
      ) : null}

      {active ? (
        <div className="fixed inset-0 z-50 flex justify-end bg-ink-950/30">
          <form className="flex h-full w-full max-w-md flex-col bg-white p-5 shadow-xl" onSubmit={(event) => { event.preventDefault(); decide([active.code], "passed"); }}>
            <div className="flex items-start justify-between gap-3">
              <div>
                <p className="text-[11px] text-ink-400">{active.code}</p>
                <h2 className="mt-1 text-[16px] font-semibold text-ink-900">{active.title}</h2>
              </div>
              <button type="button" onClick={() => setActive(null)} aria-label="关闭"><Icon name="mdi:close" /></button>
            </div>
            <p className="mt-3 text-[12px] leading-5 text-ink-500">{active.leader} · {active.rank} · {active.dept} · 申报经费 {active.budget} 万元</p>
            <label className="mt-4 block flex-1">
              <span className="field-label">审核意见</span>
              <textarea className="field-textarea h-40" value={opinion} onChange={(event) => setOpinion(event.target.value)} />
            </label>
            <div className="mt-4 flex gap-2">
              <button type="submit" className="rounded-lg bg-teal-700 px-3 py-2 text-[12px] text-white">通过</button>
              <button type="button" onClick={() => decide([active.code], "returned")} className="rounded-lg border border-gold-300 px-3 py-2 text-[12px] text-gold-800">退回</button>
            </div>
          </form>
        </div>
      ) : null}
      {toast ? <p className="fixed bottom-6 left-1/2 z-50 -translate-x-1/2 rounded-full bg-ink-900 px-4 py-2 text-[12px] text-white">{toast}</p> : null}
    </WorkspaceShell>
  );
}

function Mini({ label, value, icon }: { label: string; value: string; icon: string }) {
  return (
    <div className="rounded-xl border border-ink-200/80 bg-white p-4">
      <div className="flex items-center justify-between">
        <p className="text-[11.5px] text-ink-500">{label}</p>
        <Icon name={icon} className="text-lg text-teal-700" />
      </div>
      <p className="mt-2 text-[24px] font-semibold text-ink-900">{value}</p>
    </div>
  );
}

function StatusBadge({ status }: { status: Status }) {
  const text = status === "pending" ? "待审核" : status === "passed" ? "已通过" : "已退回";
  const tone = status === "pending" ? "bg-gold-50 text-gold-700" : status === "passed" ? "bg-teal-50 text-teal-700" : "bg-ink-100 text-ink-600";
  return <span className={`rounded-full px-2 py-0.5 text-[10px] font-medium ${tone}`}>{text}</span>;
}

"use client";

import { useMemo, useState } from "react";

import { Icon } from "@/components/icon";
import { SideNav, WorkspaceShell } from "@/components/workspace-shell";

type Status = "pending" | "blind" | "passed" | "returned";

type Review = {
  code: string;
  title: string;
  kind: "社会科学" | "自然科学";
  dept: string;
  score: string;
  recovered: string;
  budget: string;
  status: Status;
};

const reviews: Review[] = [
  { code: "SH2026-0175", title: "数字技术赋能乡村义务教育均衡发展研究", kind: "社会科学", dept: "经济管理学院", score: "86.4", recovered: "3/3", budget: "12.0", status: "pending" },
  { code: "SH2026-0198", title: "教育数字化背景下地方高校教师教学能力发展研究", kind: "社会科学", dept: "经济管理学院", score: "87.2", recovered: "3/3", budget: "10.5", status: "pending" },
  { code: "ZR2026-0044", title: "喀斯特山区特色农产品病虫害轻量化识别研究", kind: "自然科学", dept: "信息工程学院", score: "—", recovered: "1/3", budget: "18.6", status: "blind" },
  { code: "SH2026-0166", title: "民族地区高职院校产教融合绩效评价研究", kind: "社会科学", dept: "教育科学学院", score: "90.1", recovered: "3/3", budget: "8.0", status: "passed" },
  { code: "ZR2026-0038", title: "智能制造车间能耗监测方法研究", kind: "自然科学", dept: "机电工程学院", score: "71.5", recovered: "3/3", budget: "16.0", status: "returned" },
];

const publicity = [
  ["01", "SH2026-0166", "民族地区高职院校产教融合绩效评价研究", "教育科学学院", "社会科学", "8.0"],
  ["02", "SH2026-0175", "数字技术赋能乡村义务教育均衡发展研究", "经济管理学院", "社会科学", "12.0"],
  ["03", "ZR2026-0041", "山地特色水果品质无损检测研究", "现代农业学院", "自然科学", "15.0"],
  ["04", "SH2026-0182", "布依族节日文化传承与校园教育融合研究", "教育科学学院", "社会科学", "6.5"],
  ["05", "ZR2026-0046", "护理技能虚拟仿真实训资源开发", "护理学院", "自然科学", "14.2"],
  ["06", "SH2026-0190", "县域中小企业数字化转型路径研究", "经济管理学院", "社会科学", "9.8"],
  ["07", "ZR2026-0050", "新能源汽车电池热管理实验研究", "机电工程学院", "自然科学", "19.0"],
  ["08", "SH2026-0194", "乡村养老服务设施空间布局研究", "经济管理学院", "社会科学", "7.6"],
];

export function OfficeWorkspace() {
  const [section, setSection] = useState("queue");
  const [items, setItems] = useState(reviews);
  const [query, setQuery] = useState("");
  const [picked, setPicked] = useState<string[]>([]);
  const [active, setActive] = useState<Review | null>(null);
  const [opinion, setOpinion] = useState("形式审查通过，盲审均分达到立项建议线，同意进入公示。");
  const [toast, setToast] = useState("");

  const reviewNav = [
    { id: "queue", label: "复核工作台", icon: "mdi:clipboard-check-outline" },
    { id: "pending", label: "待复核", icon: "mdi:inbox-arrow-down-outline", badge: String(items.filter((item) => item.status === "pending").length) },
    { id: "blind", label: "盲审待回收", icon: "mdi:gavel" },
    { id: "passed", label: "已通过盲审", icon: "mdi:check-all" },
    { id: "returned", label: "已退回", icon: "mdi:file-undo-outline" },
    { id: "experts", label: "专家分组", icon: "mdi:account-supervisor-outline" },
    { id: "stats", label: "复核统计", icon: "mdi:chart-box-outline" },
  ];
  const publicityNav = [
    { id: "publicity", label: "公示公告", icon: "mdi:bullhorn-outline" },
    { id: "list", label: "拟立项名单", icon: "mdi:format-list-numbered" },
    { id: "objection", label: "异议处理", icon: "mdi:alert-circle-outline" },
    { id: "archive", label: "归档", icon: "mdi:archive-search-outline" },
  ];
  const top = section === "publicity" || section === "list" || section === "objection" || section === "archive" ? "publicity" : "review";
  const nav = top === "publicity" ? publicityNav : reviewNav;

  const rows = useMemo(() => {
    const scoped = section === "queue" || section === "stats" || section === "experts" ? items : items.filter((item) => item.status === section);
    return scoped.filter((item) => `${item.title}${item.code}${item.dept}`.includes(query.trim()));
  }, [items, query, section]);

  function notify(message: string) {
    setToast(message);
    window.setTimeout(() => setToast(""), 2200);
  }

  function decide(codes: string[], status: Status) {
    if (codes.length === 0) {
      notify("请先勾选项目");
      return;
    }
    setItems((current) => current.map((item) => (codes.includes(item.code) ? { ...item, status } : item)));
    setPicked([]);
    setActive(null);
    notify(status === "passed" ? "已通过复核" : "已退回");
  }

  return (
    <WorkspaceShell
      role="office"
      section={top}
      onSection={(id) => setSection(id === "publicity" ? "publicity" : "queue")}
      subtitle="科研处 · 复核与立项公示"
      sidebar={<SideNav title={top === "publicity" ? "立项公示" : "科研处复核"} items={nav} active={section} onSelect={setSection} />}
    >
      {top === "review" && section !== "experts" && section !== "stats" ? (
        <section className="rounded-xl border border-ink-200/80 bg-white shadow-sm">
          <div className="flex flex-wrap items-center gap-2 border-b border-ink-100 px-4 py-3">
            <h1 className="text-[15px] font-semibold text-ink-900">科研处复核</h1>
            <label className="ml-auto flex h-8 items-center gap-1 rounded-lg border border-ink-200 px-2 text-ink-400">
              <Icon name="mdi:magnify" />
              <input className="w-40 bg-transparent text-[12px] text-ink-800 outline-none" placeholder="项目、编号、系部" value={query} onChange={(event) => setQuery(event.target.value)} />
            </label>
            <span className="text-[11px] text-ink-400">已选 {picked.length}</span>
            <button type="button" onClick={() => decide(picked, "passed")} className="inline-flex h-7 items-center gap-1 rounded-md border border-teal-200 bg-teal-50 px-2.5 text-[10.5px] text-teal-700"><Icon name="mdi:check-all" />批量通过</button>
            <button type="button" onClick={() => decide(picked, "returned")} className="inline-flex h-7 items-center gap-1 rounded-md border border-gold-200 bg-gold-50 px-2.5 text-[10.5px] text-gold-700"><Icon name="mdi:undo-variant" />批量退回</button>
            <button type="button" onClick={() => notify("已打开批量分配终审分组")} className="inline-flex h-7 items-center gap-1 rounded-md border border-ink-200 px-2.5 text-[10.5px] text-ink-600"><Icon name="mdi:account-multiple-check-outline" />分配终审组</button>
          </div>
          <div className="overflow-x-auto">
            <table className="w-full min-w-[960px] text-left text-[12px]">
              <thead>
                <tr className="bg-ink-50/70 text-[10.5px] text-ink-500">
                  <th className="w-10 px-4 py-2.5" />
                  {["项目名称 / 编号", "类别", "系部", "盲审均分", "回收", "申请经费", "状态", "操作"].map((head) => <th key={head} className="px-3 py-2.5">{head}</th>)}
                </tr>
              </thead>
              <tbody className="divide-y divide-ink-100">
                {rows.map((item) => (
                  <tr key={item.code}>
                    <td className="px-4 py-3"><input type="checkbox" aria-label={`选择 ${item.code}`} checked={picked.includes(item.code)} onChange={() => setPicked((current) => current.includes(item.code) ? current.filter((id) => id !== item.code) : [...current, item.code])} /></td>
                    <td className="px-3 py-3"><p className="font-medium text-ink-800">{item.title}</p><p className="mt-0.5 text-[10px] text-ink-400">{item.code} · 材料 3 份</p></td>
                    <td className="px-3 py-3"><span className={`rounded border px-1.5 py-0.5 text-[10px] ${item.kind === "社会科学" ? "border-gold-200 bg-gold-50 text-gold-700" : "border-teal-200 bg-teal-50 text-teal-700"}`}>{item.kind}</span></td>
                    <td className="px-3 py-3">{item.dept}</td>
                    <td className="px-3 py-3 font-semibold text-teal-800">{item.score}</td>
                    <td className="px-3 py-3">{item.recovered}</td>
                    <td className="px-3 py-3">{item.budget} 万元</td>
                    <td className="px-3 py-3">{labelOf(item.status)}</td>
                    <td className="px-3 py-3 text-right"><button type="button" onClick={() => setActive(item)} className="rounded-md bg-teal-700 px-2.5 py-1 text-[10.5px] text-white">复核</button></td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        </section>
      ) : null}

      {section === "experts" ? (
        <section className="rounded-xl border border-ink-200/80 bg-white p-4 text-[12.5px] leading-6 text-ink-600">
          <h1 className="text-[16px] font-semibold text-ink-900">专家分组</h1>
          <p className="mt-2">终审组 A：E-016、E-021、E-028，负责社会科学。终审组 B：E-033 回避本批次。专家身份只在科研处页面显示。</p>
        </section>
      ) : null}
      {section === "stats" ? (
        <section className="rounded-xl border border-ink-200/80 bg-white p-4 text-[13px] text-ink-600">待复核 {items.filter((item) => item.status === "pending").length} 项，盲审未收齐 {items.filter((item) => item.status === "blind").length} 项，已通过 {items.filter((item) => item.status === "passed").length} 项。</section>
      ) : null}

      {top === "publicity" ? (
        <div>
          <section className="rounded-2xl bg-ink-900 p-6 text-white">
            <p className="text-[12px] tracking-wide text-teal-200">黔西南民族职业技术学院 · 科研处</p>
            <h1 className="mt-2 text-[22px] font-semibold">关于 2026 年度第一批拟立项名单的公示</h1>
            <p className="mt-2 text-[12.5px] text-ink-200">公示期 2026-11-05 至 2026-11-18 · 拟立项 40 项 · 异议请在公示期内书面提交科研处</p>
            <button type="button" onClick={() => notify("已生成公示名单导出（演示）")} className="mt-4 inline-flex items-center gap-1 rounded-lg bg-white/10 px-3 py-1.5 text-[12px]"><Icon name="mdi:download-outline" />导出名单</button>
          </section>
          {section !== "objection" && section !== "archive" ? (
            <section className="mt-4 overflow-hidden rounded-xl border border-ink-200/80 bg-white">
              <table className="w-full min-w-[760px] text-left text-[12px]">
                <thead>
                  <tr className="bg-ink-50/70 text-[10.5px] text-ink-500">
                    {["序号", "编号", "项目名称", "系部", "类别", "经费（万元）"].map((head) => <th key={head} className="px-3 py-2.5">{head}</th>)}
                  </tr>
                </thead>
                <tbody className="divide-y divide-ink-100">
                  {publicity.map((row) => (
                    <tr key={row[1]}>
                      {row.map((cell) => <td key={cell} className="px-3 py-2.5">{cell}</td>)}
                    </tr>
                  ))}
                </tbody>
              </table>
              <p className="px-4 py-3 text-[11px] text-ink-400">名单节选 8 项，公示全文共 40 项。</p>
            </section>
          ) : null}
          {section === "objection" ? <p className="mt-4 rounded-xl border border-ink-200 bg-white p-4 text-[13px] text-ink-600">公示期内暂无异议。</p> : null}
          {section === "archive" ? <p className="mt-4 rounded-xl border border-ink-200 bg-white p-4 text-[13px] text-ink-600">2025 年度第二批公示已归档，共 36 项。</p> : null}
        </div>
      ) : null}

      {active ? (
        <div className="fixed inset-0 z-50 flex justify-end bg-ink-950/30">
          <form className="h-full w-full max-w-md bg-white p-5" onSubmit={(event) => { event.preventDefault(); decide([active.code], "passed"); }}>
            <div className="flex items-start justify-between">
              <h2 className="text-[16px] font-semibold">{active.title}</h2>
              <button type="button" onClick={() => setActive(null)} aria-label="关闭"><Icon name="mdi:close" /></button>
            </div>
            <p className="mt-2 text-[12px] text-ink-500">{active.code} · {active.dept} · 盲审均分 {active.score} · 回收 {active.recovered}</p>
            <p className="mt-3 rounded-lg bg-ink-50 p-3 text-[12px] leading-5 text-ink-600">专家身份仅科研处可见：E-016、E-021、E-028。申请人页面只显示回收进度。</p>
            <label className="mt-4 block">
              <span className="field-label">复核意见</span>
              <textarea className="field-textarea" value={opinion} onChange={(event) => setOpinion(event.target.value)} />
            </label>
            <div className="mt-4 flex gap-2">
              <button type="submit" className="rounded-lg bg-teal-700 px-3 py-2 text-[12px] text-white">通过</button>
              <button type="button" className="rounded-lg border border-gold-300 px-3 py-2 text-[12px] text-gold-800" onClick={() => decide([active.code], "returned")}>退回</button>
            </div>
          </form>
        </div>
      ) : null}
      {toast ? <p className="fixed bottom-6 left-1/2 z-50 -translate-x-1/2 rounded-full bg-ink-900 px-4 py-2 text-[12px] text-white">{toast}</p> : null}
    </WorkspaceShell>
  );
}

function labelOf(status: Status) {
  if (status === "pending") return "待复核";
  if (status === "blind") return "盲审待回收";
  if (status === "passed") return "已通过";
  return "已退回";
}

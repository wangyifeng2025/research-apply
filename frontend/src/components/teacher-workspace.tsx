"use client";

import { useCallback, useEffect, useState } from "react";

import { Icon } from "@/components/icon";
import { TeacherApply } from "@/components/teacher-apply";
import { SideNav, WorkspaceShell } from "@/components/workspace-shell";
import { classLabels, fetchMine, statusLabels, type MineItem } from "@/lib/applications";
import { ApiError } from "@/lib/api";
import { readActiveLogin } from "@/lib/login";

const cycleOpen = new Date(2026, 0, 1);
const cycleClose = new Date(2026, 11, 31);
const flow = ["提交申报", "系部初审", "科研处复核", "专家匿名盲审", "立项公示"];

export function TeacherWorkspace() {
  const [section, setSection] = useState("desk");
  const [items, setItems] = useState<MineItem[]>([]);
  const [applicationId, setApplicationId] = useState<string | null>(null);
  const [filter, setFilter] = useState("all");
  const [toast, setToast] = useState("");
  const [name, setName] = useState("老师");

  const reload = useCallback(async () => {
    try {
      setItems(await fetchMine());
    } catch (error) {
      setToast(error instanceof ApiError ? error.message : "申报列表加载失败");
    }
  }, []);

  useEffect(() => {
    void reload();
    const stored = readActiveLogin();
    if (stored?.realName) setName(stored.realName);
  }, [reload]);

  const returned = items.filter((item) => item.status === "dept_returned" || item.status === "office_returned");
  const reviewing = items.filter((item) => ["submitted", "dept_approved", "office_approved", "expert_reviewing"].includes(item.status));
  const drafts = items.filter((item) => item.status === "draft");
  const established = items.filter((item) => item.status === "established");
  const done = items.filter((item) => ["established", "dept_rejected", "office_rejected", "expert_rejected"].includes(item.status));
  const socialCount = items.filter((item) => item.project_class === "social").length;
  const naturalCount = items.filter((item) => item.project_class !== "social").length;
  const visible = items.filter((item) => {
    if (filter === "draft") return item.status === "draft";
    if (filter === "reviewing") return reviewing.some((row) => row.id === item.id) || returned.some((row) => row.id === item.id);
    if (filter === "done") return done.some((row) => row.id === item.id);
    return true;
  });
  const focus = [...reviewing, ...returned, ...items].find((item) => item.status !== "draft") ?? items[0];
  const today = new Date();
  const daysLeft = Math.max(0, Math.ceil((cycleClose.getTime() - today.getTime()) / 86_400_000));
  const span = cycleClose.getTime() - cycleOpen.getTime();
  const elapsed = Math.min(100, Math.max(0, Math.round(((today.getTime() - cycleOpen.getTime()) / span) * 100)));

  function openNew() {
    setApplicationId(null);
    setSection("apply");
  }

  function openItem(id: string) {
    setApplicationId(id);
    setSection("apply");
  }

  function ping(text: string) {
    setToast(text);
    window.setTimeout(() => setToast(""), 2600);
  }

  const nav = [
    { id: "desk", label: "工作台", icon: "mdi:view-dashboard-outline" },
    { id: "projects", label: "我的申报", icon: "mdi:folder-text-outline", badge: String(items.length) },
    { id: "apply", label: "项目填报", icon: "mdi:file-document-edit-outline" },
    { id: "track", label: "审核进度", icon: "mdi:timeline-clock-outline" },
    { id: "history", label: "历史申报复用", icon: "mdi:history" },
    { id: "notice", label: "立项公示查询", icon: "mdi:bullhorn-outline", divider: true },
    { id: "message", label: "消息中心", icon: "mdi:message-text-outline", badge: returned.length ? String(returned.length) : undefined, gold: returned.length > 0 },
  ];

  return (
    <WorkspaceShell
      role="teacher"
      section={section === "apply" ? "apply" : "desk"}
      onSection={(id) => setSection(id === "apply" ? "apply" : "desk")}
      subtitle="2026 年度申报周期"
      sidebar={
        <SideNav
          title="教师工作区"
          items={nav}
          active={section}
          onSelect={setSection}
          footer={
            <div className="mb-2 rounded-xl bg-gradient-to-br from-ink-900 to-ink-800 p-3.5 text-white">
              <div className="flex items-center gap-2 text-[11px] text-ink-300">
                <Icon name="mdi:calendar-clock-outline" className="text-sm text-teal-300" />
                2026 年度申报
              </div>
              <p className="mt-2 text-[12.5px] font-medium">距离申报截止还有 {daysLeft} 天</p>
              <div className="mt-2.5 h-1.5 overflow-hidden rounded-full bg-white/15">
                <div className="h-full rounded-full bg-teal-400" style={{ width: `${elapsed}%` }} />
              </div>
              <button type="button" onClick={() => ping("申报指南以科研处下发的正式文件为准")} className="mt-3 flex w-full items-center justify-center gap-1 rounded-md bg-white/10 py-1.5 text-[11px] text-ink-100">
                <Icon name="mdi:file-pdf-box" className="text-sm" />
                申报说明
              </button>
            </div>
          }
        />
      }
    >
      {section === "apply" ? (
        <TeacherApply
          key={applicationId ?? "new"}
          applicationId={applicationId}
          onOpen={openItem}
          onClose={() => {
            setApplicationId(null);
            setSection("projects");
            void reload();
          }}
          onChanged={() => void reload()}
        />
      ) : null}

      {section === "desk" || section === "projects" ? (
        <div>
          {section === "desk" ? (
            <>
              <div className="flex flex-col justify-between gap-3 sm:flex-row sm:items-end">
                <div>
                  <div className="flex items-center gap-1.5 text-[11px] text-ink-400">
                    <span>首页</span>
                    <Icon name="mdi:chevron-right" className="text-xs" />
                    <span className="text-ink-500">教师工作台</span>
                  </div>
                  <h1 className="mt-1.5 text-[22px] font-semibold tracking-tight text-ink-900">{name}，{hello(today)}</h1>
                  <p className="mt-1 text-[12.5px] text-ink-500">
                    {today.getFullYear()} 年 {today.getMonth() + 1} 月 {today.getDate()} 日 · 星期{"日一二三四五六"[today.getDay()]} · 您有 <span className="font-medium text-gold-700">{returned.length} 项</span> 待办需要处理
                  </p>
                </div>
                <div className="flex items-center gap-2">
                  <button type="button" onClick={() => setSection("history")} className="inline-flex h-9 items-center gap-1.5 rounded-lg border border-ink-200 bg-white px-3 text-[12px] font-medium text-ink-600">
                    <Icon name="mdi:content-duplicate" className="text-base" />历史复用
                  </button>
                  <button type="button" onClick={openNew} className="inline-flex h-9 items-center gap-1.5 rounded-lg bg-teal-700 px-3.5 text-[12px] font-medium text-white shadow-sm">
                    <Icon name="mdi:plus" className="text-base" />新建申报
                  </button>
                </div>
              </div>
              <div className="mt-4 flex items-start gap-2.5 rounded-xl border border-teal-200/70 bg-teal-50/70 px-3.5 py-2.5 text-[11.5px] leading-5 text-teal-900">
                <Icon name="mdi:bullhorn-variant-outline" className="mt-0.5 shrink-0 text-base text-teal-600" />
                <p className="flex-1"><span className="font-medium">申报提醒：</span>2026 年度申报周期至 12 月 31 日。草稿可以继续填写，已提交项目可以查看当前环节。本页数字来自你本人的申报书。</p>
              </div>
              <div className="mt-4 grid grid-cols-2 gap-3 xl:grid-cols-4">
                <Stat label="在审项目" value={String(reviewing.length)} icon="mdi:file-search-outline" note={`${items.filter((item) => item.status === "expert_reviewing").length} 项正在专家盲审`} />
                <Stat label="待修改 / 退回" value={String(returned.length)} icon="mdi:file-alert-outline" warn note={returned.length > 0 ? "请打开申报书按退回意见修改" : "目前没有退回"} />
                <Stat label="已立项项目" value={String(established.length)} icon="mdi:check-decagram-outline" note="立项金额在公示环节公布" />
                <Stat label="本年度申报" value={String(items.length)} icon="mdi:chart-timeline-variant" note={`社科 ${socialCount} 项 · 自科与双高 ${naturalCount} 项`} />
              </div>
            </>
          ) : (
            <div className="flex flex-wrap items-end justify-between gap-3">
              <div>
                <h1 className="text-[22px] font-semibold text-ink-900">我的申报</h1>
                <p className="mt-1 text-[12.5px] text-ink-500">列表来自你本人负责的申报书。</p>
              </div>
              <button type="button" onClick={openNew} className="inline-flex h-9 items-center gap-1.5 rounded-lg bg-teal-700 px-3.5 text-[12px] font-medium text-white">
                <Icon name="mdi:plus" className="text-base" />新建申报
              </button>
            </div>
          )}

          <div className={`grid grid-cols-1 gap-4 ${section === "desk" ? "mt-4 xl:grid-cols-3" : "mt-4"}`}>
            <div className="space-y-4 xl:col-span-2">
              {section === "desk" ? (
                <section className="rounded-xl border border-ink-200/80 bg-white shadow-sm">
                  <div className="flex items-center justify-between border-b border-ink-100 px-4 py-3.5">
                    <div className="flex items-center gap-2">
                      <h2 className="text-[14px] font-semibold text-ink-900">待办任务</h2>
                      <span className="rounded-full bg-gold-50 px-2 py-0.5 text-[10px] font-medium text-gold-700">{returned.length} 项待处理</span>
                    </div>
                    <button type="button" onClick={() => setSection("track")} className="text-[11px] font-medium text-teal-700">查看全部</button>
                  </div>
                  {returned.length === 0 ? (
                    <p className="px-4 py-6 text-[12.5px] text-ink-400">没有被退回的申报书。</p>
                  ) : returned.map((item) => (
                    <div key={item.id} className="flex items-start gap-3 border-t border-ink-100 px-4 py-3.5">
                      <span className="mt-0.5 flex h-8 w-8 shrink-0 items-center justify-center rounded-lg bg-gold-50 text-gold-600"><Icon name="mdi:file-undo-outline" className="text-lg" /></span>
                      <div className="min-w-0 flex-1">
                        <p className="text-[12.5px] font-medium text-ink-800">《{item.title || "未命名申报书"}》被退回</p>
                        <p className="mt-1 text-[11.5px] text-ink-500">{statusLabels[item.status]}。请打开申报书修改后重新提交。</p>
                      </div>
                      <button type="button" onClick={() => openItem(item.id)} className="inline-flex shrink-0 items-center rounded-md border border-ink-200 px-2.5 py-1.5 text-[11px] font-medium text-ink-600">去修改</button>
                    </div>
                  ))}
                </section>
              ) : null}
              <ProjectTable
                items={visible}
                total={items.length}
                drafts={drafts.length}
                reviewing={reviewing.length + returned.length}
                done={done.length}
                filter={filter}
                onFilter={setFilter}
                onOpen={openItem}
              />
              {section === "desk" ? <HistoryCards items={established} onOpen={openItem} onAll={() => setSection("history")} /> : null}
            </div>
            {section === "desk" ? (
              <div className="space-y-4">
                <ProgressCard item={focus} onOpen={() => focus && openItem(focus.id)} />
                <section className="rounded-xl border border-ink-200/80 bg-white p-4 shadow-sm">
                  <h2 className="text-[14px] font-semibold text-ink-900">开始新的申报</h2>
                  <div className="mt-3 space-y-2.5">
                    <StartCard icon="mdi:book-open-page-variant-outline" tone="gold" title="社会科学类申报" text="人文社科、经济管理、教育学等方向，按社会科学类申报书填写" onClick={openNew} />
                    <StartCard icon="mdi:atom" tone="teal" title="自然科学类申报" text="工程技术、信息科学等方向，双高专项也使用这份申报书" onClick={openNew} />
                  </div>
                </section>
                <section className="rounded-xl border border-ink-200/80 bg-white shadow-sm">
                  <div className="flex items-center justify-between border-b border-ink-100 px-4 py-3.5">
                    <h2 className="text-[14px] font-semibold text-ink-900">消息提醒</h2>
                    <button type="button" onClick={() => setSection("message")} className="text-[11px] text-teal-700">查看全部</button>
                  </div>
                  {returned.length === 0 ? <p className="px-4 py-5 text-[12px] text-ink-400">目前没有退回消息。</p> : returned.map((item) => (
                    <button key={item.id} type="button" onClick={() => openItem(item.id)} className="flex w-full gap-2.5 border-t border-ink-100 px-4 py-3 text-left">
                      <span className="mt-1 h-1.5 w-1.5 shrink-0 rounded-full bg-gold-500" />
                      <span>
                        <span className="block text-[11.5px] font-medium text-ink-800">{statusLabels[item.status]}</span>
                        <span className="mt-0.5 block text-[10.5px] leading-4 text-ink-500">《{item.title || "未命名申报书"}》需要修改后重新提交。</span>
                      </span>
                    </button>
                  ))}
                </section>
                <section className="rounded-xl border border-teal-200/70 bg-teal-50/50 p-4">
                  <div className="flex items-center gap-2">
                    <span className="flex h-7 w-7 items-center justify-center rounded-lg bg-teal-600 text-white"><Icon name="mdi:calendar-clock-outline" className="text-base" /></span>
                    <h2 className="text-[13px] font-semibold text-ink-900">填报咨询</h2>
                  </div>
                  <p className="mt-2 text-[11px] leading-5 text-ink-600">贵州省兴义市丰都街道民航大道 66 号行政楼 316 室。邮编 562400。qxnzyky@163.com · 0859-3655091</p>
                </section>
              </div>
            ) : null}
          </div>
        </div>
      ) : null}

      {section === "track" ? (
        <section className="space-y-3">
          <h1 className="text-[22px] font-semibold text-ink-900">审核进度</h1>
          {items.length === 0 ? <p className="text-[13px] text-ink-500">提交之后，这里会显示每份申报书的当前环节。</p> : items.map((item) => (
            <article key={item.id} className="rounded-xl border border-ink-200/80 bg-white p-4">
              <div className="flex items-start justify-between gap-3">
                <div>
                  <p className="text-[14px] font-medium text-ink-900">{item.title || "未命名申报书"}</p>
                  <p className="mt-1 text-[12px] text-ink-500">{classLabels[item.project_class]} · {statusLabels[item.status]}</p>
                </div>
                <button type="button" onClick={() => openItem(item.id)} className="text-[12px] text-teal-700">打开</button>
              </div>
              <Flow status={item.status} />
            </article>
          ))}
        </section>
      ) : null}

      {section === "history" ? (
        <section>
          <h1 className="text-[22px] font-semibold text-ink-900">历史申报复用</h1>
          <p className="mt-1 text-[12.5px] text-ink-500">已立项的申报书可以打开查看。复制成新草稿还没有开通，避免把往年内容直接写成今年的申报。</p>
          <div className="mt-4"><HistoryCards items={established} onOpen={openItem} /></div>
        </section>
      ) : null}

      {section === "notice" ? (
        <section className="rounded-xl border border-ink-200 bg-white p-5">
          <h1 className="text-[18px] font-semibold text-ink-900">立项公示查询</h1>
          <p className="mt-2 text-[13px] leading-6 text-ink-600">公示名单由科研处发布。你已立项的项目有 {established.length} 项，可在「我的申报」里查看状态。</p>
        </section>
      ) : null}

      {section === "message" ? (
        <section className="rounded-xl border border-ink-200 bg-white p-4 text-[13px] text-ink-600">
          <h1 className="text-[18px] font-semibold text-ink-900">消息中心</h1>
          {returned.length === 0 ? <p className="mt-3">目前没有退回消息。</p> : returned.map((item) => (
            <button key={item.id} type="button" onClick={() => openItem(item.id)} className="mt-3 block w-full rounded-lg border border-ink-100 px-3 py-2 text-left">
              {item.title || "未命名申报书"} · {statusLabels[item.status]}
            </button>
          ))}
        </section>
      ) : null}
      {toast ? <p className="fixed bottom-6 left-1/2 z-50 -translate-x-1/2 rounded-lg bg-ink-900 px-4 py-2.5 text-[12px] text-white">{toast}</p> : null}
    </WorkspaceShell>
  );
}

function hello(date: Date) {
  const hour = date.getHours();
  if (hour < 11) return "上午好";
  if (hour < 18) return "下午好";
  return "晚上好";
}

function stageName(status: string) {
  if (status === "draft") return "填写申报书";
  if (status === "submitted" || status === "dept_returned" || status === "dept_rejected") return "系部初审";
  if (status === "dept_approved" || status === "office_returned" || status === "office_rejected") return "科研处复核";
  if (status === "office_approved" || status === "expert_reviewing" || status === "expert_rejected") return "专家盲审";
  if (status === "established") return "立项公示";
  return statusLabels[status] ?? status;
}

function stageIndex(status: string) {
  if (status === "draft") return -1;
  if (status === "submitted" || status === "dept_returned" || status === "dept_rejected") return 1;
  if (status === "dept_approved" || status === "office_returned" || status === "office_rejected") return 2;
  if (status === "office_approved" || status === "expert_reviewing" || status === "expert_rejected") return 3;
  if (status === "established") return 5;
  return 0;
}

function stamp(value: string) {
  return value.slice(0, 16).replace("T", " ");
}

function Stat({ label, value, icon, note, warn }: { label: string; value: string; icon: string; note: string; warn?: boolean }) {
  return (
    <div className="rounded-xl border border-ink-200/80 bg-white p-4 shadow-sm">
      <div className="flex items-start justify-between">
        <div>
          <p className="text-[11.5px] text-ink-500">{label}</p>
          <p className="mt-2 text-[26px] leading-none font-semibold tracking-tight text-ink-900">{value}<span className="ml-1 text-[11px] font-normal text-ink-400">项</span></p>
        </div>
        <span className={`flex h-9 w-9 items-center justify-center rounded-lg ${warn ? "bg-gold-50 text-gold-600" : "bg-teal-50 text-teal-600"}`}><Icon name={icon} className="text-[19px]" /></span>
      </div>
      <p className="mt-3 text-[11px] text-ink-400">{note}</p>
    </div>
  );
}

function ProjectTable({ items, total, drafts, reviewing, done, filter, onFilter, onOpen }: { items: MineItem[]; total: number; drafts: number; reviewing: number; done: number; filter: string; onFilter: (id: string) => void; onOpen: (id: string) => void }) {
  const tabs = [
    ["all", `全部 ${total}`],
    ["reviewing", `审核中 ${reviewing}`],
    ["draft", `草稿 ${drafts}`],
    ["done", `已完成 ${done}`],
  ];
  return (
    <section className="rounded-xl border border-ink-200/80 bg-white shadow-sm">
      <div className="flex flex-wrap items-center justify-between gap-2 border-b border-ink-100 px-4 py-3.5">
        <div className="flex items-center gap-2">
          <h2 className="text-[14px] font-semibold text-ink-900">我的申报</h2>
          <span className="text-[11px] text-ink-400">2026 年度</span>
        </div>
        <div className="flex items-center gap-1 rounded-md bg-ink-50 p-0.5 text-[11px]">
          {tabs.map(([id, label]) => (
            <button key={id} type="button" onClick={() => onFilter(id ?? "all")} className={`rounded px-2.5 py-1 ${filter === id ? "bg-white font-medium text-ink-800 shadow-sm" : "text-ink-500"}`}>{label}</button>
          ))}
        </div>
      </div>
      <div className="overflow-x-auto">
        <table className="w-full min-w-[720px] text-left">
          <thead>
            <tr className="bg-ink-50/70 text-[10.5px] font-medium text-ink-500">
              <th className="px-4 py-2.5">项目名称</th>
              <th className="px-3 py-2.5">类别</th>
              <th className="px-3 py-2.5">当前环节</th>
              <th className="px-3 py-2.5">状态</th>
              <th className="px-3 py-2.5">更新时间</th>
              <th className="px-4 py-2.5 text-right">操作</th>
            </tr>
          </thead>
          <tbody className="divide-y divide-ink-100 text-[12px]">
            {items.length === 0 ? (
              <tr><td colSpan={6} className="px-4 py-8 text-center text-ink-400">还没有申报书。点右上角新建一份草稿。</td></tr>
            ) : items.map((item) => (
              <tr key={item.id} className="hover:bg-ink-50/50">
                <td className="px-4 py-3">
                  <p className="font-medium text-ink-800">{item.title || "未命名申报书"}</p>
                  <p className="mt-0.5 text-[10.5px] text-ink-400">{item.status === "draft" ? "草稿" : statusLabels[item.status]} · {stamp(item.updated_at)}</p>
                </td>
                <td className="px-3 py-3"><span className={`rounded border px-1.5 py-0.5 text-[10.5px] ${item.project_class === "natural" ? "border-teal-200 bg-teal-50 text-teal-700" : "border-ink-200 bg-ink-50 text-ink-600"}`}>{classLabels[item.project_class]}</span></td>
                <td className="px-3 py-3 text-ink-600">{stageName(item.status)}</td>
                <td className="px-3 py-3"><StatusPill status={item.status} /></td>
                <td className="px-3 py-3 text-[11px] text-ink-400">{stamp(item.updated_at)}</td>
                <td className="px-4 py-3 text-right">
                  <button type="button" onClick={() => onOpen(item.id)} className="text-[11px] font-medium text-teal-700">{item.status === "draft" || item.status.endsWith("returned") ? "继续填写" : "查看"}</button>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </section>
  );
}

function StatusPill({ status }: { status: string }) {
  const returned = status.endsWith("returned");
  const live = ["submitted", "dept_approved", "office_approved", "expert_reviewing"].includes(status);
  const tone = status === "established" ? "bg-ink-900 text-white" : returned ? "bg-gold-50 text-gold-700" : live ? "bg-teal-50 text-teal-700" : "bg-ink-100 text-ink-600";
  return <span className={`rounded-full px-2 py-1 text-[10.5px] font-medium ${tone}`}>{statusLabels[status] ?? status}</span>;
}

function HistoryCards({ items, onOpen, onAll }: { items: MineItem[]; onOpen: (id: string) => void; onAll?: () => void }) {
  return (
    <section className="rounded-xl border border-ink-200/80 bg-white shadow-sm">
      <div className="flex items-center justify-between border-b border-ink-100 px-4 py-3.5">
        <div>
          <h2 className="text-[14px] font-semibold text-ink-900">历史申报复用</h2>
          <p className="mt-0.5 text-[11px] text-ink-400">已立项项目可以打开查看。复制为新草稿尚未开通。</p>
        </div>
        {onAll ? <button type="button" onClick={onAll} className="text-[11px] font-medium text-teal-700">全部记录</button> : null}
      </div>
      {items.length === 0 ? (
        <p className="px-4 py-6 text-[12.5px] text-ink-400">还没有已立项的申报可以复用。</p>
      ) : (
        <div className="grid gap-2.5 p-3 sm:grid-cols-3">
          {items.map((item) => (
            <div key={item.id} className="rounded-lg border border-ink-200 p-3">
              <div className="flex items-center justify-between">
                <span className="text-[10.5px] text-ink-400">{classLabels[item.project_class]}</span>
                <span className="rounded bg-ink-900 px-1.5 py-0.5 text-[9.5px] text-white">已立项</span>
              </div>
              <p className="mt-2 line-clamp-2 text-[12px] font-medium leading-5 text-ink-800">{item.title || "未命名申报书"}</p>
              <button type="button" onClick={() => onOpen(item.id)} className="mt-2.5 flex w-full items-center justify-center gap-1 rounded-md border border-ink-200 py-1.5 text-[11px] text-ink-600">
                <Icon name="mdi:content-duplicate" className="text-sm" />查看
              </button>
            </div>
          ))}
        </div>
      )}
    </section>
  );
}

function ProgressCard({ item, onOpen }: { item?: MineItem; onOpen: () => void }) {
  return (
    <section className="rounded-xl border border-ink-200/80 bg-white shadow-sm">
      <div className="flex items-center justify-between border-b border-ink-100 px-4 py-3.5">
        <h2 className="text-[14px] font-semibold text-ink-900">当前申报进度</h2>
        {item ? <span className="rounded-full bg-teal-50 px-2 py-0.5 text-[10px] font-medium text-teal-700">{statusLabels[item.status]}</span> : null}
      </div>
      <div className="p-4">
        {item ? (
          <>
            <p className="text-[12.5px] font-medium leading-5 text-ink-800">{item.title || "未命名申报书"}</p>
            <p className="mt-1.5 text-[10.5px] text-ink-400">{classLabels[item.project_class]} · 更新于 {stamp(item.updated_at)}</p>
            {item.status === "draft" ? <p className="mt-3 text-[12px] text-ink-600">仍在填写申报书，尚未提交。</p> : null}
            <div className="mt-4"><Flow status={item.status} /></div>
            <button type="button" onClick={onOpen} className="mt-3 flex h-8 w-full items-center justify-center gap-1 rounded-lg border border-ink-200 text-[11.5px] font-medium text-ink-600">打开这份申报书<Icon name="mdi:arrow-right" className="text-sm" /></button>
          </>
        ) : <p className="text-[12.5px] text-ink-400">还没有申报书。</p>}
      </div>
    </section>
  );
}

function Flow({ status }: { status: string }) {
  const current = stageIndex(status);
  return (
    <div>
      {flow.map((label, index) => {
        const done = current > index;
        const active = current === index;
        return (
          <div key={label} className="flex gap-3">
            <div className="flex flex-col items-center">
              <span className={`flex h-6 w-6 items-center justify-center rounded-full ${done || (active && status === "established") ? "bg-teal-600 text-white" : active ? "bg-teal-50 text-teal-700 ring-1 ring-teal-300" : "border-2 border-ink-200 bg-white text-ink-400"}`}>
                {done || (active && status === "established") ? <Icon name="mdi:check" className="text-sm" /> : <span className="h-1.5 w-1.5 rounded-full bg-current" />}
              </span>
              {index < flow.length - 1 ? <span className={`my-1 w-px flex-1 ${done ? "bg-teal-200" : "bg-ink-200"}`} /> : null}
            </div>
            <div className="pb-4">
              <p className={`text-[12px] font-medium ${done || active ? "text-ink-800" : "text-ink-500"}`}>{label}{active ? " · 当前" : ""}</p>
            </div>
          </div>
        );
      })}
    </div>
  );
}

function StartCard({ icon, tone, title, text, onClick }: { icon: string; tone: "gold" | "teal"; title: string; text: string; onClick: () => void }) {
  return (
    <button type="button" onClick={onClick} className="flex w-full items-start gap-3 rounded-lg border border-ink-200 p-3 text-left hover:border-teal-300 hover:bg-teal-50/40">
      <span className={`flex h-9 w-9 shrink-0 items-center justify-center rounded-lg ${tone === "gold" ? "bg-gold-50 text-gold-600" : "bg-teal-50 text-teal-600"}`}><Icon name={icon} className="text-lg" /></span>
      <span className="flex-1">
        <span className="block text-[12.5px] font-medium text-ink-800">{title}</span>
        <span className="mt-0.5 block text-[10.5px] leading-4 text-ink-500">{text}</span>
      </span>
      <Icon name="mdi:arrow-right" className="mt-1 text-base text-ink-300" />
    </button>
  );
}

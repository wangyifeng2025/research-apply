"use client";

import { useMemo, useState, type ReactNode } from "react";

import { Icon } from "@/components/icon";
import { SideNav, WorkspaceShell } from "@/components/workspace-shell";

const sections = [
  { id: "stats", label: "运行概览", icon: "mdi:chart-box-outline" },
  { id: "departments", label: "部门管理", icon: "mdi:office-building-cog-outline" },
  { id: "users", label: "用户管理", icon: "mdi:account-multiple-outline", badge: "24" },
  { id: "roles", label: "角色权限", icon: "mdi:shield-account-outline" },
  { id: "templates", label: "申报模板", icon: "mdi:file-document-edit-outline" },
  { id: "experts", label: "专家库", icon: "mdi:account-supervisor-outline" },
  { id: "flow", label: "审核流程", icon: "mdi:sitemap-outline" },
  { id: "logs", label: "操作日志", icon: "mdi:text-box-search-outline" },
];

const initialDepartments = [
  { id: "D01", name: "黔西南民族职业技术学院", code: "XY000", parent: "—", leader: "张建国", phone: "0859-3210000", members: 612, status: "启用" },
  { id: "D101", name: "经济管理学院", code: "XY101", parent: "学院", leader: "李雪梅", phone: "0859-3210101", members: 168, status: "启用" },
  { id: "D10101", name: "公共管理系", code: "XY10101", parent: "经济管理学院", leader: "王立群", phone: "0859-3210111", members: 42, status: "启用" },
  { id: "D10105", name: "工商管理系", code: "XY10102", parent: "经济管理学院", leader: "赵明远", phone: "0859-3210112", members: 46, status: "启用" },
  { id: "D102", name: "现代农业学院", code: "XY102", parent: "学院", leader: "周成林", phone: "0859-3210201", members: 96, status: "启用" },
  { id: "D103", name: "机电工程学院", code: "XY103", parent: "学院", leader: "钱志刚", phone: "0859-3210301", members: 112, status: "启用" },
  { id: "D104", name: "护理学院", code: "XY104", parent: "学院", leader: "郑丽华", phone: "0859-3210401", members: 118, status: "启用" },
  { id: "D107", name: "信息工程学院", code: "XY107", parent: "学院", leader: "许俊杰", phone: "0859-3210701", members: 72, status: "启用" },
  { id: "D108", name: "科研处", code: "GL001", parent: "学院", leader: "韩志远", phone: "0859-3210088", members: 12, status: "启用" },
  { id: "D109", name: "信息中心", code: "GL002", parent: "学院", leader: "曹磊", phone: "0859-3210099", members: 8, status: "启用" },
];

const users = [
  { name: "张建国", jobNo: "T2001001", title: "教授 / 院长", dept: "学院", role: "系统管理员", phone: "138****6601", email: "zhangjg@qxnzy.edu.cn", login: "2026-10-20 09:12", status: "正常" },
  { name: "赵明远", jobNo: "T2001007", title: "讲师", dept: "经济管理学院", role: "教师", phone: "139****1104", email: "zhaomy@qxnzy.edu.cn", login: "2026-10-19 11:02", status: "正常" },
  { name: "李文博", jobNo: "T2001011", title: "副教授", dept: "经济管理学院", role: "系部负责人", phone: "137****2208", email: "liwb@qxnzy.edu.cn", login: "2026-10-20 08:16", status: "正常" },
  { name: "王海宁", jobNo: "G2001008", title: "副处长", dept: "科研处", role: "科研处管理员", phone: "136****4410", email: "wanghn@qxnzy.edu.cn", login: "2026-10-20 09:01", status: "正常" },
  { name: "周谨", jobNo: "G2001003", title: "科员", dept: "科研处", role: "系统管理员", phone: "135****7780", email: "zhoujin@qxnzy.edu.cn", login: "2026-10-20 09:20", status: "正常" },
  { name: "陈霞", jobNo: "T2007012", title: "讲师", dept: "信息工程学院", role: "教师", phone: "138****2291", email: "chenxia@qxnzy.edu.cn", login: "2026-10-12 15:40", status: "正常" },
  { name: "杨慧", jobNo: "T2001006", title: "讲师", dept: "公共管理系", role: "教师", phone: "138****9932", email: "yanghui@qxnzy.edu.cn", login: "—", status: "待激活" },
  { name: "马成龙", jobNo: "T2006001", title: "副教授 / 院长", dept: "材料工程学院", role: "教师", phone: "135****7781", email: "macl@qxnzy.edu.cn", login: "2026-08-30 09:00", status: "停用" },
  { name: "许俊杰", jobNo: "T2007001", title: "副教授", dept: "信息工程学院", role: "评审专家", phone: "136****6608", email: "xujj@qxnzy.edu.cn", login: "2026-10-19 20:31", status: "正常" },
  { name: "韩志远", jobNo: "G2001001", title: "处长", dept: "科研处", role: "科研处管理员", phone: "139****0022", email: "hanzy@qxnzy.edu.cn", login: "2026-10-20 08:58", status: "正常" },
];

const experts = [
  { code: "E-016", name: "匿名专家 E-016", field: "乡村旅游 / 区域经济", title: "教授", pool: "社会科学", status: "在库" },
  { code: "E-021", name: "许俊杰", field: "农业物联网", title: "副教授", pool: "自然科学", status: "在库" },
  { code: "E-028", name: "罗春花", field: "畜牧兽医", title: "教授", pool: "自然科学", status: "在库" },
  { code: "E-033", name: "邓海燕", field: "护理教育", title: "教授", pool: "社会科学", status: "回避本批次" },
];

const logs = [
  ["2026-10-20 09:20", "周谨", "登录成功", "系统管理"],
  ["2026-10-20 09:05", "刘思远", "开启 2026 年度第一批申报", "申报批次"],
  ["2026-10-19 16:42", "李文博", "退回 SH2026-0187", "系部初审"],
  ["2026-10-19 11:18", "王海宁", "分配盲审专家 3 人", "科研处复核"],
];

const permissions = ["创建申报", "提交申报", "系部审核", "科研处复核", "专家评分", "公示发布", "用户管理", "模板配置"];
const roleMatrix: Record<string, boolean[]> = {
  教师: [true, true, false, false, false, false, false, false],
  系部负责人: [false, false, true, false, false, false, false, false],
  科研处管理员: [false, false, false, true, false, true, false, true],
  评审专家: [false, false, false, false, true, false, false, false],
  系统管理员: [true, true, true, true, true, true, true, true],
};

const years = ["2021", "2022", "2023", "2024", "2025", "2026"];
const socialTrend = [142, 156, 171, 183, 196, 186];
const naturalTrend = [108, 121, 134, 147, 158, 140];

export function AdminWorkspace() {
  const [section, setSection] = useState("departments");
  const [query, setQuery] = useState("");
  const [roleFilter, setRoleFilter] = useState("全部角色");
  const [departments, setDepartments] = useState(initialDepartments);
  const [editor, setEditor] = useState(false);
  const [draft, setDraft] = useState({ name: "", code: "", leader: "", phone: "" });
  const [toast, setToast] = useState("");
  const [selected, setSelected] = useState<string[]>([]);

  const visibleUsers = useMemo(
    () => users.filter((user) => (roleFilter === "全部角色" || user.role === roleFilter) && `${user.name}${user.jobNo}${user.dept}`.includes(query.trim())),
    [query, roleFilter],
  );
  const visibleDepartments = departments.filter((item) => `${item.name}${item.code}${item.leader}`.includes(query.trim()));

  function notify(message: string) {
    setToast(message);
    window.setTimeout(() => setToast(""), 2200);
  }

  return (
    <WorkspaceShell role="admin" section="console" onSection={() => setSection("stats")} subtitle="系统管理 · 组织、账号与流程" sidebar={<SideNav title="系统管理" items={sections} active={section} onSelect={setSection} />}>
      {section === "stats" ? (
        <div>
          <h1 className="text-[22px] font-semibold text-ink-900">运行概览</h1>
          <div className="mt-4 grid grid-cols-2 gap-3 xl:grid-cols-4">
            {[
              ["在册用户", "612"],
              ["启用部门", String(departments.length)],
              ["本批次申报", "186"],
              ["在库专家", "48"],
            ].map(([label, value]) => (
              <div key={label} className="rounded-xl border border-ink-200/80 bg-white p-4">
                <p className="text-[11.5px] text-ink-500">{label}</p>
                <p className="mt-2 text-[26px] font-semibold text-ink-900">{value}</p>
              </div>
            ))}
          </div>
          <section className="mt-4 rounded-xl border border-ink-200/80 bg-white p-4">
            <h2 className="text-[14px] font-semibold text-ink-900">近六年申报量</h2>
            <div className="mt-4 flex h-40 items-end gap-3">
              {years.map((year, index) => (
                <div key={year} className="flex flex-1 flex-col items-center gap-1">
                  <div className="flex h-32 w-full items-end justify-center gap-1">
                    <span className="w-3 rounded-t bg-gold-400" style={{ height: `${((socialTrend[index] ?? 0) / 220) * 100}%` }} title={`社科 ${socialTrend[index]}`} />
                    <span className="w-3 rounded-t bg-teal-500" style={{ height: `${((naturalTrend[index] ?? 0) / 220) * 100}%` }} title={`自科 ${naturalTrend[index]}`} />
                  </div>
                  <span className="text-[10px] text-ink-400">{year}</span>
                </div>
              ))}
            </div>
            <p className="mt-2 text-[11px] text-ink-400">金色为社会科学，青色为自然科学。2026 年社科 186、自科 140。</p>
          </section>
        </div>
      ) : null}

      {section === "departments" ? (
        <Panel title="部门管理" action="新增部门" onAction={() => setEditor(true)}>
          <Search value={query} onChange={setQuery} placeholder="搜索部门、编码、负责人" />
          <div className="overflow-x-auto">
            <table className="w-full min-w-[860px] text-left text-[12px]">
              <thead>
                <tr className="bg-ink-50/70 text-[10.5px] text-ink-500">
                  {["名称", "编码", "上级", "负责人", "电话", "成员数", "状态", "操作"].map((head) => <th key={head} className="px-3 py-2.5">{head}</th>)}
                </tr>
              </thead>
              <tbody className="divide-y divide-ink-100">
                {visibleDepartments.map((item) => (
                  <tr key={item.id}>
                    <td className="px-3 py-2.5 font-medium text-ink-800">{item.name}</td>
                    <td className="px-3 py-2.5 text-ink-500">{item.code}</td>
                    <td className="px-3 py-2.5">{item.parent}</td>
                    <td className="px-3 py-2.5">{item.leader}</td>
                    <td className="px-3 py-2.5">{item.phone}</td>
                    <td className="px-3 py-2.5">{item.members}</td>
                    <td className="px-3 py-2.5"><span className="rounded-full bg-teal-50 px-2 py-0.5 text-[10px] text-teal-700">{item.status}</span></td>
                    <td className="px-3 py-2.5">
                      <button type="button" className="text-teal-700" onClick={() => notify(`已打开 ${item.name}`)}>编辑</button>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        </Panel>
      ) : null}

      {section === "users" ? (
        <Panel title="用户管理" action="新增用户" onAction={() => notify("用户写入接口尚未接通，当前只在页面演示")}>
          <div className="flex flex-wrap items-center gap-2 border-b border-ink-100 px-4 py-3">
            <Search value={query} onChange={setQuery} placeholder="姓名、工号、部门" />
            <select className="field-input w-auto" value={roleFilter} onChange={(event) => setRoleFilter(event.target.value)}>
              {["全部角色", "教师", "系部负责人", "科研处管理员", "评审专家", "系统管理员"].map((item) => <option key={item}>{item}</option>)}
            </select>
            <button type="button" className="text-[12px] text-ink-500" onClick={() => { setSelected([]); notify("已清空选择"); }}>批量停用</button>
          </div>
          <div className="overflow-x-auto">
            <table className="w-full min-w-[980px] text-left text-[12px]">
              <thead>
                <tr className="bg-ink-50/70 text-[10.5px] text-ink-500">
                  <th className="px-3 py-2.5" />
                  {["姓名 / 工号", "职称", "部门", "角色", "手机", "邮箱", "最近登录", "状态"].map((head) => <th key={head} className="px-3 py-2.5">{head}</th>)}
                </tr>
              </thead>
              <tbody className="divide-y divide-ink-100">
                {visibleUsers.map((user) => (
                  <tr key={user.jobNo}>
                    <td className="px-3 py-2.5"><input type="checkbox" checked={selected.includes(user.jobNo)} onChange={() => setSelected((current) => current.includes(user.jobNo) ? current.filter((id) => id !== user.jobNo) : [...current, user.jobNo])} /></td>
                    <td className="px-3 py-2.5"><p className="font-medium">{user.name}</p><p className="text-[10px] text-ink-400">{user.jobNo}</p></td>
                    <td className="px-3 py-2.5">{user.title}</td>
                    <td className="px-3 py-2.5">{user.dept}</td>
                    <td className="px-3 py-2.5">{user.role}</td>
                    <td className="px-3 py-2.5">{user.phone}</td>
                    <td className="px-3 py-2.5">{user.email}</td>
                    <td className="px-3 py-2.5 text-ink-400">{user.login}</td>
                    <td className="px-3 py-2.5"><span className={`rounded-full px-2 py-0.5 text-[10px] ${user.status === "正常" ? "bg-teal-50 text-teal-700" : user.status === "停用" ? "bg-red-50 text-red-600" : "bg-gold-50 text-gold-700"}`}>{user.status}</span></td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
          <p className="px-4 py-3 text-[11px] text-ink-400">已选 {selected.length} 人 · 演示数据，停用不会写入数据库</p>
        </Panel>
      ) : null}

      {section === "roles" ? (
        <section className="grid gap-4 lg:grid-cols-[220px_1fr]">
          <aside className="rounded-xl border border-ink-200/80 bg-white p-3">
            {Object.keys(roleMatrix).map((role) => <p key={role} className="rounded-lg px-3 py-2 text-[13px] text-ink-700 hover:bg-ink-50">{role}</p>)}
          </aside>
          <div className="overflow-x-auto rounded-xl border border-ink-200/80 bg-white">
            <table className="w-full text-left text-[12px]">
              <thead>
                <tr className="bg-ink-50/70 text-[10.5px] text-ink-500">
                  <th className="px-3 py-2.5">角色</th>
                  {permissions.map((item) => <th key={item} className="px-2 py-2.5">{item}</th>)}
                </tr>
              </thead>
              <tbody>
                {Object.entries(roleMatrix).map(([role, flags]) => (
                  <tr key={role} className="border-t border-ink-100">
                    <td className="px-3 py-2 font-medium">{role}</td>
                    {flags.map((flag, index) => <td key={permissions[index]} className="px-2 py-2 text-center">{flag ? <Icon name="mdi:check" className="inline text-teal-700" /> : "—"}</td>)}
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        </section>
      ) : null}

      {section === "templates" ? (
        <div className="grid gap-3 lg:grid-cols-2">
          <TemplateCard code="RW-2026-V3" title="社会科学类项目申报书" chapters="七章：基本情况、近五年成果、设计论证、研究基础、预期成果、经费预算、推荐人意见" />
          <TemplateCard code="ZR-2026-V4" title="自然科学类项目申报书" chapters="八章：基本情况、研究内容、技术经济指标、计划进度、仪器清单、经费预算、推荐人、部门意见" />
        </div>
      ) : null}

      {section === "experts" ? (
        <Panel title="专家库" action="新增专家" onAction={() => notify("专家库仍是页面演示")}>
          <table className="w-full text-left text-[12px]">
            <thead>
              <tr className="bg-ink-50/70 text-[10.5px] text-ink-500">
                {["编号", "姓名", "学科方向", "职称", "库别", "状态"].map((head) => <th key={head} className="px-3 py-2.5">{head}</th>)}
              </tr>
            </thead>
            <tbody className="divide-y divide-ink-100">
              {experts.map((item) => (
                <tr key={item.code}>
                  <td className="px-3 py-2.5">{item.code}</td>
                  <td className="px-3 py-2.5 font-medium">{item.name}</td>
                  <td className="px-3 py-2.5">{item.field}</td>
                  <td className="px-3 py-2.5">{item.title}</td>
                  <td className="px-3 py-2.5">{item.pool}</td>
                  <td className="px-3 py-2.5">{item.status}</td>
                </tr>
              ))}
            </tbody>
          </table>
        </Panel>
      ) : null}

      {section === "flow" ? (
        <section className="rounded-xl border border-ink-200/80 bg-white p-5">
          <h1 className="text-[16px] font-semibold text-ink-900">审核流程</h1>
          <ol className="mt-4 space-y-3">
            {["教师填写并提交", "系部初审", "科研处形式复核", "专家盲审（身份对申请人不可见）", "科研处汇总并公示"].map((item, index) => (
              <li key={item} className="flex items-center gap-3 rounded-lg border border-ink-100 px-3 py-3 text-[13px]">
                <span className="flex h-7 w-7 items-center justify-center rounded-full bg-teal-700 text-[12px] text-white">{index + 1}</span>
                {item}
                <span className="ml-auto text-[11px] text-teal-700">启用</span>
              </li>
            ))}
          </ol>
          <button type="button" onClick={() => notify("配置已保存在本页，新配置将在下一个流程实例生效")} className="mt-4 rounded-lg bg-teal-700 px-3 py-2 text-[12px] text-white">保存配置</button>
        </section>
      ) : null}

      {section === "logs" ? (
        <Panel title="操作日志">
          <table className="w-full text-left text-[12px]">
            <tbody className="divide-y divide-ink-100">
              {logs.map((item) => (
                <tr key={item.join("-")}>
                  {item.map((cell) => <td key={cell} className="px-4 py-3">{cell}</td>)}
                </tr>
              ))}
            </tbody>
          </table>
        </Panel>
      ) : null}

      {editor ? (
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-ink-950/40 p-4">
          <form
            className="w-full max-w-md rounded-2xl bg-white p-5"
            onSubmit={(event) => {
              event.preventDefault();
              setDepartments([{ id: `D${Date.now()}`, name: draft.name, code: draft.code, parent: "学院", leader: draft.leader, phone: draft.phone, members: 0, status: "启用" }, ...departments]);
              setEditor(false);
              setDraft({ name: "", code: "", leader: "", phone: "" });
              notify("部门已加到本页列表");
            }}
          >
            <div className="flex items-center justify-between">
              <h2 className="text-[16px] font-semibold">新增部门</h2>
              <button type="button" onClick={() => setEditor(false)} aria-label="关闭"><Icon name="mdi:close" /></button>
            </div>
            <div className="mt-4 space-y-3">
              <label className="block"><span className="field-label">名称</span><input required className="field-input" value={draft.name} onChange={(event) => setDraft({ ...draft, name: event.target.value })} /></label>
              <label className="block"><span className="field-label">编码</span><input required className="field-input" value={draft.code} onChange={(event) => setDraft({ ...draft, code: event.target.value })} /></label>
              <label className="block"><span className="field-label">负责人</span><input required className="field-input" value={draft.leader} onChange={(event) => setDraft({ ...draft, leader: event.target.value })} /></label>
              <label className="block"><span className="field-label">电话</span><input className="field-input" value={draft.phone} onChange={(event) => setDraft({ ...draft, phone: event.target.value })} /></label>
            </div>
            <button type="submit" className="mt-4 rounded-lg bg-teal-700 px-3 py-2 text-[12px] text-white">保存</button>
          </form>
        </div>
      ) : null}
      {toast ? <p className="fixed bottom-6 left-1/2 z-50 -translate-x-1/2 rounded-full bg-ink-900 px-4 py-2 text-[12px] text-white">{toast}</p> : null}
    </WorkspaceShell>
  );
}

function Panel({ title, action, onAction, children }: { title: string; action?: string; onAction?: () => void; children: ReactNode }) {
  return (
    <section className="rounded-xl border border-ink-200/80 bg-white shadow-sm">
      <div className="flex items-center justify-between border-b border-ink-100 px-4 py-3.5">
        <h1 className="text-[16px] font-semibold text-ink-900">{title}</h1>
        {action ? <button type="button" onClick={onAction} className="inline-flex h-8 items-center gap-1 rounded-lg bg-teal-700 px-3 text-[12px] text-white"><Icon name="mdi:plus" />{action}</button> : null}
      </div>
      {children}
    </section>
  );
}

function Search({ value, onChange, placeholder }: { value: string; onChange: (value: string) => void; placeholder: string }) {
  return (
    <label className="flex h-8 items-center gap-1 rounded-lg border border-ink-200 px-2 text-ink-400">
      <Icon name="mdi:magnify" />
      <input className="w-48 bg-transparent text-[12px] text-ink-800 outline-none" value={value} placeholder={placeholder} onChange={(event) => onChange(event.target.value)} />
    </label>
  );
}

function TemplateCard({ code, title, chapters }: { code: string; title: string; chapters: string }) {
  return (
    <article className="rounded-xl border border-ink-200/80 bg-white p-4">
      <p className="text-[11px] text-teal-700">{code}</p>
      <h2 className="mt-1 text-[15px] font-semibold text-ink-900">{title}</h2>
      <p className="mt-2 text-[12px] leading-5 text-ink-500">{chapters}</p>
    </article>
  );
}

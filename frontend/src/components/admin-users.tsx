"use client";

import { useState, type FormEvent } from "react";

import { AdminPanel, AdminStat, Field, messageOf } from "@/components/admin-ui";
import { Icon } from "@/components/icon";
import { UserEditorDialog } from "@/components/user-editor";
import { ConfirmDialog } from "@/components/ui/alert-dialog";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Checkbox } from "@/components/ui/checkbox";
import { DialogBody, DialogFooter, FormDialog } from "@/components/ui/dialog";
import { Input } from "@/components/ui/input";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import { StickyTable } from "@/components/ui/sticky-table";
import { createUser, deleteUser, resetUserPassword, roleLabel, roleOptions, updateUser, type AdminUser, type Department, type RoleCode, type UserInput } from "@/lib/admin";

const PAGE_SIZE = 10;

export function AdminUsers({ users, departments, onChanged, onNotify }: { users: AdminUser[]; departments: Department[]; onChanged: () => Promise<void>; onNotify: (message: string) => void }) {
  const [filters, setFilters] = useState({ keyword: "", department: "all", role: "all", status: "all" });
  const [selected, setSelected] = useState<string[]>([]);
  const [page, setPage] = useState(1);
  const [busy, setBusy] = useState("");
  const [editorOpen, setEditorOpen] = useState(false);
  const [editing, setEditing] = useState<AdminUser | null>(null);
  const [resetTarget, setResetTarget] = useState<AdminUser | null>(null);
  const [password, setPassword] = useState("");
  const [confirmPassword, setConfirmPassword] = useState("");
  const [pendingDelete, setPendingDelete] = useState<AdminUser[] | null>(null);
  const keyword = filters.keyword.trim().toLocaleLowerCase();
  const rows = users.filter((user) => {
    const text = `${user.real_name}${user.username}${user.employee_no ?? ""}${user.email ?? ""}`.toLocaleLowerCase();
    return (!keyword || text.includes(keyword))
      && (filters.department === "all" || user.department_id === filters.department)
      && (filters.role === "all" || user.roles.includes(filters.role as RoleCode))
      && (filters.status === "all" || (filters.status === "active" ? user.is_active : !user.is_active));
  });
  const pageCount = Math.max(1, Math.ceil(rows.length / PAGE_SIZE));
  const currentPage = Math.min(page, pageCount);
  const pageRows = rows.slice((currentPage - 1) * PAGE_SIZE, currentPage * PAGE_SIZE);
  const allSelected = pageRows.length > 0 && pageRows.every((user) => selected.includes(user.id));
  const teacherCount = users.filter((user) => user.roles.includes("teacher")).length;
  const reviewerCount = users.filter((user) => user.roles.some((role) => ["department_reviewer", "research_office", "admin"].includes(role))).length;
  const inactiveCount = users.filter((user) => !user.is_active).length;

  function changeFilter(key: keyof typeof filters, value: string) {
    setFilters((current) => ({ ...current, [key]: value }));
    setPage(1);
  }

  function openEditor(user?: AdminUser) {
    setEditing(user ?? null);
    setEditorOpen(true);
  }

  async function refresh() {
    await onChanged();
    setSelected([]);
  }

  async function submit(input: UserInput, initialPassword: string) {
    setBusy("editor");
    try {
      if (editing) await updateUser(editing.id, input);
      else await createUser({ ...input, password: initialPassword });
      await refresh();
      setEditorOpen(false);
      onNotify(editing ? "用户信息已更新" : "用户已创建");
    } catch (error) {
      onNotify(messageOf(error));
    } finally {
      setBusy("");
    }
  }

  async function toggle(user: AdminUser) {
    setBusy(user.id);
    try {
      await updateUser(user.id, { is_active: !user.is_active });
      await refresh();
      onNotify(user.is_active ? "账号已停用" : "账号已启用");
    } catch (error) {
      onNotify(messageOf(error));
    } finally {
      setBusy("");
    }
  }

  async function removeSelected() {
    if (!pendingDelete) return;
    setBusy("delete");
    let failure = "";
    try {
      for (const user of pendingDelete) await deleteUser(user.id);
    } catch (error) {
      failure = messageOf(error);
    }
    await refresh().catch(() => undefined);
    setPendingDelete(null);
    setBusy("");
    onNotify(failure ? `删除已停止，请核对列表：${failure}` : "用户已删除");
  }

  async function batch(action: "enable" | "disable") {
    if (selected.length === 0) {
      onNotify("请先勾选需要操作的用户");
      return;
    }
    setBusy("batch");
    let failure = "";
    try {
      for (const id of selected) await updateUser(id, { is_active: action === "enable" });
    } catch (error) {
      failure = messageOf(error);
    }
    await refresh().catch(() => undefined);
    setBusy("");
    onNotify(failure ? `批量操作已停止，请核对列表：${failure}` : action === "enable" ? "选中账号已启用" : "选中账号已停用");
  }

  async function resetPassword(event: FormEvent) {
    event.preventDefault();
    if (!resetTarget) return;
    if (password !== confirmPassword) {
      onNotify("两次输入的密码不一致");
      return;
    }
    setBusy("password");
    try {
      await resetUserPassword(resetTarget.id, password);
      setResetTarget(null);
      setPassword("");
      setConfirmPassword("");
      onNotify("密码已重置");
    } catch (error) {
      onNotify(messageOf(error));
    } finally {
      setBusy("");
    }
  }

  return (
    <div className="mt-4">
      <div className="grid grid-cols-2 gap-3 lg:grid-cols-4">
        <AdminStat label="用户总数" value={users.length} unit="人" />
        <AdminStat label="教师账号" value={teacherCount} unit="人" />
        <AdminStat label="审核 / 管理账号" value={reviewerCount} unit="人" />
        <AdminStat label="停用账号" value={inactiveCount} unit="人" gold />
      </div>
      <AdminPanel className="mt-4 overflow-hidden">
        <div className="flex flex-wrap items-center gap-2 border-b border-ink-100 px-4 py-3">
          <label className="relative">
            <Icon name="mdi:magnify" className="absolute top-1/2 left-2.5 -translate-y-1/2 text-ink-400" />
            <Input className="w-52 pl-7" value={filters.keyword} placeholder="搜索姓名 / 工号 / 邮箱" onChange={(event) => changeFilter("keyword", event.target.value)} />
          </label>
          <Select value={filters.department} onValueChange={(value) => changeFilter("department", value)}>
            <SelectTrigger aria-label="部门筛选" className="w-[150px]"><SelectValue /></SelectTrigger>
            <SelectContent>
              <SelectItem value="all">全部部门</SelectItem>
              {departments.map((department) => <SelectItem key={department.id} value={department.id}>{department.name}</SelectItem>)}
            </SelectContent>
          </Select>
          <Select value={filters.role} onValueChange={(value) => changeFilter("role", value)}>
            <SelectTrigger aria-label="角色筛选" className="w-[150px]"><SelectValue /></SelectTrigger>
            <SelectContent>
              <SelectItem value="all">全部角色</SelectItem>
              {roleOptions.map((role) => <SelectItem key={role.code} value={role.code}>{role.label}</SelectItem>)}
            </SelectContent>
          </Select>
          <Select value={filters.status} onValueChange={(value) => changeFilter("status", value)}>
            <SelectTrigger aria-label="状态筛选" className="w-[120px]"><SelectValue /></SelectTrigger>
            <SelectContent>
              <SelectItem value="all">全部状态</SelectItem>
              <SelectItem value="active">正常</SelectItem>
              <SelectItem value="inactive">停用</SelectItem>
            </SelectContent>
          </Select>
          <div className="ml-auto flex flex-wrap items-center gap-1.5">
            <span className={selected.length ? "text-[12px] font-medium text-teal-700" : "text-[12px] text-ink-400"}>{selected.length ? `已选择 ${selected.length} 个用户` : "未选择用户"}</span>
            <Button variant="outline" disabled={busy === "batch"} onClick={() => void batch("enable")}>批量启用</Button>
            <Button variant="warning" disabled={busy === "batch"} onClick={() => void batch("disable")}>批量停用</Button>
            <Button variant="destructive" onClick={() => { if (selected.length === 0) onNotify("请先勾选需要操作的用户"); else setPendingDelete(users.filter((user) => selected.includes(user.id))); }}>批量删除</Button>
            <Button onClick={() => openEditor()}><Icon name="mdi:plus" />新增用户</Button>
          </div>
        </div>
        <StickyTable
          rows={pageRows}
          rowKey={(user) => user.id}
          empty="没有符合筛选条件的用户"
          muted={(user) => !user.is_active}
          columns={[
            {
              header: <Checkbox aria-label="全选本页用户" checked={allSelected} onCheckedChange={() => setSelected((current) => allSelected ? current.filter((id) => !pageRows.some((user) => user.id === id)) : [...new Set([...current, ...pageRows.map((user) => user.id)])])} />,
              width: 52,
              cell: (user) => <Checkbox aria-label={`选择用户${user.real_name}`} checked={selected.includes(user.id)} onCheckedChange={() => setSelected((current) => current.includes(user.id) ? current.filter((id) => id !== user.id) : [...current, user.id])} />,
            },
            {
              header: "姓名 / 工号",
              width: 168,
              cell: (user) => <div><p className="font-medium text-ink-900">{user.real_name}</p><p className="mt-0.5 text-[11px] text-ink-400">{user.employee_no || user.username}</p></div>,
            },
            { header: "职称 / 职务", width: 150, cell: (user) => [user.professional_title, user.admin_title].filter(Boolean).join(" / ") || "—" },
            { header: "所属部门", width: 140, cell: (user) => user.department_name || "未分配部门" },
            { header: "角色", width: 168, cell: (user) => <div className="flex flex-wrap gap-1">{user.roles.map((role) => <Badge key={role} tone="teal">{roleLabel(role)}</Badge>)}</div> },
            { header: "手机号码", width: 130, cell: (user) => user.phone || "—" },
            { header: "邮箱", width: 190, cell: (user) => user.email || "—" },
            { header: "状态", width: 88, cell: (user) => <Badge tone={user.is_active ? "teal" : "neutral"}>{user.is_active ? "正常" : "停用"}</Badge> },
            {
              header: "操作",
              width: 248,
              align: "right",
              cell: (user) => (
                <div className="flex justify-end gap-1">
                  <Button variant="subtle" size="sm" onClick={() => openEditor(user)}>编辑</Button>
                  <Button variant="ghost" size="sm" onClick={() => { setResetTarget(user); setPassword(""); setConfirmPassword(""); }}>重置密码</Button>
                  <Button variant={user.is_active ? "warning" : "subtle"} size="sm" disabled={busy === user.id} onClick={() => void toggle(user)}>{user.is_active ? "停用" : "启用"}</Button>
                  <Button variant="destructive" size="sm" onClick={() => setPendingDelete([user])}>删除</Button>
                </div>
              ),
            },
          ]}
        />
        <div className="flex flex-wrap items-center justify-between gap-2 border-t border-ink-100 px-4 py-3 text-[12px] text-ink-400">
          <span>共 {rows.length} 个用户{rows.length !== users.length ? `，账号总数 ${users.length}` : ""}</span>
          <div className="flex items-center gap-1">
            <Button variant="outline" size="icon" className="h-7 w-7" aria-label="上一页" disabled={currentPage === 1} onClick={() => setPage(currentPage - 1)}><Icon name="mdi:chevron-left" /></Button>
            {Array.from({ length: pageCount }, (_, index) => index + 1).map((number) => <Button key={number} variant={number === currentPage ? "default" : "outline"} size="icon" className="h-7 w-7" onClick={() => setPage(number)}>{number}</Button>)}
            <Button variant="outline" size="icon" className="h-7 w-7" aria-label="下一页" disabled={currentPage === pageCount} onClick={() => setPage(currentPage + 1)}><Icon name="mdi:chevron-right" /></Button>
          </div>
        </div>
      </AdminPanel>
      {editorOpen ? <UserEditorDialog key={editing?.id ?? "new"} open user={editing} departments={departments} busy={busy === "editor"} onClose={() => setEditorOpen(false)} onSubmit={submit} /> : null}
      <FormDialog open={resetTarget !== null} title={resetTarget ? `重置密码 · ${resetTarget.real_name}` : "重置密码"} description="请设置至少 8 位的新密码。" onClose={() => setResetTarget(null)}>
        <form className="flex min-h-0 flex-1 flex-col" onSubmit={(event) => void resetPassword(event)}>
          <DialogBody className="space-y-3">
            <Field label="新密码" required><Input required minLength={8} type="password" autoComplete="new-password" value={password} onChange={(event) => setPassword(event.target.value)} /></Field>
            <Field label="再次输入" required><Input required minLength={8} type="password" autoComplete="new-password" value={confirmPassword} onChange={(event) => setConfirmPassword(event.target.value)} /></Field>
          </DialogBody>
          <DialogFooter>
            <Button variant="outline" onClick={() => setResetTarget(null)}>取消</Button>
            <Button type="submit" disabled={busy === "password"}>{busy === "password" ? "保存中…" : "重置密码"}</Button>
          </DialogFooter>
        </form>
      </FormDialog>
      <ConfirmDialog open={pendingDelete !== null} title="删除用户" description={pendingDelete && pendingDelete.length > 1 ? `确定删除选中的 ${pendingDelete.length} 个账号吗？已关联申报或审核记录的用户不能删除。` : `确定删除“${pendingDelete?.[0]?.real_name ?? ""}（${pendingDelete?.[0]?.username ?? ""}）”吗？已关联申报或审核记录的用户不能删除。`} confirmLabel="删除" busy={busy === "delete"} onConfirm={() => void removeSelected()} onClose={() => setPendingDelete(null)} />
    </div>
  );
}

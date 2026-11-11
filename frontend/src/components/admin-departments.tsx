"use client";

import { useState, type FormEvent } from "react";

import { AdminPanel, AdminStat, Field, messageOf } from "@/components/admin-ui";
import { Icon } from "@/components/icon";
import { ConfirmDialog } from "@/components/ui/alert-dialog";
import { Button } from "@/components/ui/button";
import { DialogBody, DialogFooter, FormDialog } from "@/components/ui/dialog";
import { Input } from "@/components/ui/input";
import { StickyTable } from "@/components/ui/sticky-table";
import { createDepartment, deleteDepartment, updateDepartment, type Department } from "@/lib/admin";

export function AdminDepartments({ departments, onChanged, onNotify }: { departments: Department[]; onChanged: () => Promise<void>; onNotify: (message: string) => void }) {
  const [keyword, setKeyword] = useState("");
  const [editing, setEditing] = useState<Department | null>(null);
  const [editorOpen, setEditorOpen] = useState(false);
  const [form, setForm] = useState({ name: "", code: "" });
  const [busy, setBusy] = useState(false);
  const [pendingDelete, setPendingDelete] = useState<Department | null>(null);
  const normalized = keyword.trim().toLocaleLowerCase();
  const rows = departments.filter((department) => `${department.name}${department.code ?? ""}`.toLocaleLowerCase().includes(normalized));
  const memberCount = departments.reduce((total, department) => total + department.member_count, 0);

  function openEditor(department?: Department) {
    setEditing(department ?? null);
    setForm({ name: department?.name ?? "", code: department?.code ?? "" });
    setEditorOpen(true);
  }

  async function submit(event: FormEvent) {
    event.preventDefault();
    setBusy(true);
    try {
      const input = { name: form.name, code: form.code.trim() || null };
      if (editing) await updateDepartment(editing.id, input);
      else await createDepartment(input);
      await onChanged();
      setEditorOpen(false);
      onNotify(editing ? "部门已更新" : "部门已创建");
    } catch (error) {
      onNotify(messageOf(error));
    } finally {
      setBusy(false);
    }
  }

  async function remove() {
    if (!pendingDelete) return;
    setBusy(true);
    try {
      await deleteDepartment(pendingDelete.id);
      await onChanged();
      setPendingDelete(null);
      onNotify("部门已删除");
    } catch (error) {
      onNotify(messageOf(error));
    } finally {
      setBusy(false);
    }
  }

  return (
    <div className="mt-4">
      <div className="grid grid-cols-2 gap-3 lg:grid-cols-3">
        <AdminStat label="部门总数" value={departments.length} unit="个" />
        <AdminStat label="关联用户" value={memberCount} unit="人" />
        <AdminStat label="已设部门编码" value={departments.filter((item) => item.code).length} unit="个" />
      </div>
      <AdminPanel className="mt-4 overflow-hidden">
        <div className="flex flex-wrap items-center gap-2 border-b border-ink-100 px-4 py-3">
          <label className="relative">
            <Icon name="mdi:magnify" className="absolute top-1/2 left-2.5 -translate-y-1/2 text-ink-400" />
            <Input className="w-64 pl-7" value={keyword} placeholder="搜索部门名称 / 编码" onChange={(event) => setKeyword(event.target.value)} />
          </label>
          <span className="text-[12px] text-ink-400">部门是扁平结构，没有上级部门</span>
          <Button className="ml-auto" onClick={() => openEditor()}><Icon name="mdi:plus" />新增部门</Button>
        </div>
        <StickyTable
          rows={rows}
          rowKey={(department) => department.id}
          empty="未找到匹配的部门"
          columns={[
            { header: "部门名称", width: 240, cell: (department) => <span className="font-medium text-ink-900">{department.name}</span> },
            { header: "部门编码", width: 160, cell: (department) => department.code || "—" },
            { header: "成员数", width: 120, cell: (department) => department.member_count },
            {
              header: "操作",
              width: 140,
              align: "right",
              cell: (department) => (
                <div className="flex justify-end gap-1">
                  <Button variant="subtle" size="sm" onClick={() => openEditor(department)}>编辑</Button>
                  <Button variant="destructive" size="sm" disabled={busy} onClick={() => setPendingDelete(department)}>删除</Button>
                </div>
              ),
            },
          ]}
        />
        <div className="flex justify-between border-t border-ink-100 px-4 py-3 text-[12px] text-ink-400">
          <span>有关联用户或申报书的部门不能删除</span>
          <span>共 {departments.length} 个部门{keyword ? `，筛选出 ${rows.length} 个` : ""}</span>
        </div>
      </AdminPanel>
      <FormDialog open={editorOpen} title={editing ? "编辑部门" : "新增部门"} description="部门为扁平结构，部门编码全院唯一。" onClose={() => setEditorOpen(false)}>
        <form className="flex min-h-0 flex-1 flex-col" onSubmit={(event) => void submit(event)}>
          <DialogBody className="grid gap-3 sm:grid-cols-2">
            <Field label="部门名称" required><Input required maxLength={100} value={form.name} placeholder="如 公共管理系" onChange={(event) => setForm({ ...form, name: event.target.value })} /></Field>
            <Field label="部门编码"><Input maxLength={32} value={form.code} placeholder="如 D101" onChange={(event) => setForm({ ...form, code: event.target.value })} /></Field>
          </DialogBody>
          <DialogFooter>
            <Button variant="outline" onClick={() => setEditorOpen(false)}>取消</Button>
            <Button type="submit" disabled={busy}>{busy ? "保存中…" : "保存部门"}</Button>
          </DialogFooter>
        </form>
      </FormDialog>
      <ConfirmDialog open={pendingDelete !== null} title="删除部门" description={`确定删除“${pendingDelete?.name ?? ""}”吗？已有用户或申报书的部门不能删除。`} confirmLabel="删除" busy={busy} onConfirm={() => void remove()} onClose={() => setPendingDelete(null)} />
    </div>
  );
}

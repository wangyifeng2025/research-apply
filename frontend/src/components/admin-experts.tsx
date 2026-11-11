"use client";

import { useState } from "react";

import { AdminPanel, AdminStat, messageOf } from "@/components/admin-ui";
import { Icon } from "@/components/icon";
import { UserEditorDialog } from "@/components/user-editor";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { StickyTable } from "@/components/ui/sticky-table";
import { createUser, roleLabel, updateUser, type AdminUser, type Department, type UserInput } from "@/lib/admin";

export function AdminExperts({ users, departments, onChanged, onNotify }: { users: AdminUser[]; departments: Department[]; onChanged: () => Promise<void>; onNotify: (message: string) => void }) {
  const experts = users.filter((user) => user.roles.includes("expert"));
  const [keyword, setKeyword] = useState("");
  const [editorOpen, setEditorOpen] = useState(false);
  const [editing, setEditing] = useState<AdminUser | null>(null);
  const [busy, setBusy] = useState(false);
  const normalized = keyword.trim().toLocaleLowerCase();
  const rows = experts.filter((user) => `${user.real_name}${user.username}${user.research_expertise ?? ""}${user.department_name ?? ""}`.toLocaleLowerCase().includes(normalized));
  const activeCount = experts.filter((user) => user.is_active).length;

  function openEditor(user?: AdminUser) {
    setEditing(user ?? null);
    setEditorOpen(true);
  }

  async function submit(input: UserInput, password: string) {
    setBusy(true);
    try {
      const roles = input.roles.includes("expert") ? input.roles : [...input.roles, "expert" as const];
      if (editing) await updateUser(editing.id, { ...input, roles });
      else await createUser({ ...input, roles, password });
      await onChanged();
      setEditorOpen(false);
      onNotify(editing ? "专家资料已更新" : "专家账号已创建");
    } catch (error) {
      onNotify(messageOf(error));
    } finally {
      setBusy(false);
    }
  }

  return (
    <div className="mt-4">
      <div className="grid grid-cols-2 gap-3 lg:grid-cols-3">
        <AdminStat label="专家账号" value={experts.length} unit="人" icon="mdi:account-supervisor-outline" />
        <AdminStat label="启用中" value={activeCount} unit="人" />
        <AdminStat label="停用" value={experts.length - activeCount} unit="人" gold />
      </div>
      <AdminPanel className="mt-4 overflow-hidden">
        <div className="flex flex-wrap items-center gap-2 border-b border-ink-100 px-4 py-3">
          <label className="relative">
            <Icon name="mdi:magnify" className="absolute top-1/2 left-2.5 -translate-y-1/2 text-ink-400" />
            <Input className="w-64 pl-7" value={keyword} placeholder="搜索姓名 / 专长 / 部门" onChange={(event) => setKeyword(event.target.value)} />
          </label>
          <span className="text-[12px] text-ink-400">名单来自带“{roleLabel("expert")}”角色的用户。指派和回避还没有接口。</span>
          <Button className="ml-auto" onClick={() => openEditor()}><Icon name="mdi:plus" />新增专家</Button>
        </div>
        <StickyTable
          rows={rows}
          rowKey={(user) => user.id}
          empty="还没有评审专家"
          muted={(user) => !user.is_active}
          columns={[
            { header: "姓名", width: 140, cell: (user) => <span className="font-medium text-ink-900">{user.real_name}</span> },
            { header: "账号", width: 140, cell: (user) => user.username },
            { header: "部门", width: 160, cell: (user) => user.department_name || "未分配部门" },
            { header: "职称", width: 120, cell: (user) => user.professional_title || "—" },
            { header: "研究专长", width: 220, cell: (user) => user.research_expertise || "—" },
            { header: "状态", width: 88, cell: (user) => <Badge tone={user.is_active ? "teal" : "neutral"}>{user.is_active ? "正常" : "停用"}</Badge> },
            { header: "操作", width: 88, align: "right", cell: (user) => <Button variant="subtle" size="sm" onClick={() => openEditor(user)}>编辑</Button> },
          ]}
        />
      </AdminPanel>
      {editorOpen ? <UserEditorDialog key={editing?.id ?? "new-expert"} open user={editing} departments={departments} busy={busy} defaultRoles={["expert"]} title={editing ? "编辑专家" : "新增专家"} onClose={() => setEditorOpen(false)} onSubmit={submit} /> : null}
    </div>
  );
}

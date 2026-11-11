"use client";

import { useState, type FormEvent } from "react";

import { Field } from "@/components/admin-ui";
import { Button } from "@/components/ui/button";
import { Checkbox } from "@/components/ui/checkbox";
import { DialogBody, DialogFooter, FormDialog } from "@/components/ui/dialog";
import { Input } from "@/components/ui/input";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import { roleOptions, type AdminUser, type Department, type RoleCode, type UserInput } from "@/lib/admin";

type UserForm = UserInput & { password: string };

function fromUser(user: AdminUser | null, defaultRoles: RoleCode[]): UserForm {
  if (!user) {
    return {
      username: "",
      password: "",
      real_name: "",
      employee_no: null,
      gender: null,
      ethnicity: null,
      admin_title: null,
      professional_title: null,
      research_expertise: null,
      education: null,
      degree: null,
      phone: null,
      email: null,
      department_id: null,
      is_active: true,
      roles: defaultRoles,
    };
  }
  return { ...user, password: "" };
}

export function UserEditorDialog({
  open,
  user,
  departments,
  busy,
  title,
  defaultRoles = ["teacher"],
  onClose,
  onSubmit,
}: {
  open: boolean;
  user: AdminUser | null;
  departments: Department[];
  busy: boolean;
  title?: string;
  defaultRoles?: RoleCode[];
  onClose: () => void;
  onSubmit: (input: UserInput, password: string) => Promise<void>;
}) {
  const [form, setForm] = useState<UserForm>(() => fromUser(user, defaultRoles));

  function setText(key: "employee_no" | "phone" | "email" | "professional_title" | "admin_title" | "research_expertise" | "gender" | "ethnicity" | "education" | "degree", value: string) {
    setForm((current) => ({ ...current, [key]: value || null }));
  }

  async function submit(event: FormEvent) {
    event.preventDefault();
    const { password, ...input } = form;
    await onSubmit(input, password);
  }

  return (
    <FormDialog
      open={open}
      wide
      title={title ?? (user ? "编辑用户信息" : "新增系统用户")}
      description={user ? "修改角色或部门后，账号权限按新配置生效。" : "请设置账号资料、角色和初始密码。"}
      onClose={onClose}
    >
      <form className="flex min-h-0 flex-1 flex-col" onSubmit={(event) => void submit(event)}>
        <DialogBody className="grid gap-3 sm:grid-cols-2">
          <Field label="姓名" required><Input required value={form.real_name} onChange={(event) => setForm({ ...form, real_name: event.target.value })} /></Field>
          <Field label="工号"><Input value={form.employee_no ?? ""} onChange={(event) => setText("employee_no", event.target.value)} /></Field>
          <Field label="登录账号" required><Input required value={form.username} onChange={(event) => setForm({ ...form, username: event.target.value })} /></Field>
          {user ? null : <Field label="初始密码" required><Input required minLength={8} type="password" autoComplete="new-password" value={form.password} onChange={(event) => setForm({ ...form, password: event.target.value })} /></Field>}
          <Field label="手机号码"><Input value={form.phone ?? ""} onChange={(event) => setText("phone", event.target.value)} /></Field>
          <Field label="校园邮箱"><Input type="email" value={form.email ?? ""} onChange={(event) => setText("email", event.target.value)} /></Field>
          <Field label="所属部门">
            <Select value={form.department_id ?? "none"} onValueChange={(value) => setForm({ ...form, department_id: value === "none" ? null : value })}>
              <SelectTrigger><SelectValue /></SelectTrigger>
              <SelectContent>
                <SelectItem value="none">未分配</SelectItem>
                {departments.map((department) => <SelectItem key={department.id} value={department.id}>{department.name}</SelectItem>)}
              </SelectContent>
            </Select>
          </Field>
          <Field label="账号状态">
            <Select value={form.is_active ? "active" : "inactive"} onValueChange={(value) => setForm({ ...form, is_active: value === "active" })}>
              <SelectTrigger><SelectValue /></SelectTrigger>
              <SelectContent>
                <SelectItem value="active">正常</SelectItem>
                <SelectItem value="inactive">停用</SelectItem>
              </SelectContent>
            </Select>
          </Field>
          <Field label="职称"><Input value={form.professional_title ?? ""} onChange={(event) => setText("professional_title", event.target.value)} /></Field>
          <Field label="行政职务"><Input value={form.admin_title ?? ""} onChange={(event) => setText("admin_title", event.target.value)} /></Field>
          <div className="sm:col-span-2">
            <span className="mb-1 block text-[12px] font-medium text-ink-700">角色（可多选）</span>
            <div className="flex flex-wrap gap-2">
              {roleOptions.map((role) => {
                const checked = form.roles.includes(role.code);
                return (
                  <label key={role.code} className={`flex items-center gap-2 rounded-lg border px-2.5 py-2 text-[12px] ${checked ? "border-teal-300 bg-teal-50 text-teal-800" : "border-ink-200"}`}>
                    <Checkbox checked={checked} onCheckedChange={() => setForm({ ...form, roles: checked ? form.roles.filter((item) => item !== role.code) : [...form.roles, role.code] })} />
                    {role.label}
                  </label>
                );
              })}
            </div>
          </div>
          <Field label="研究专长"><Input value={form.research_expertise ?? ""} onChange={(event) => setText("research_expertise", event.target.value)} /></Field>
          <Field label="性别"><Input value={form.gender ?? ""} onChange={(event) => setText("gender", event.target.value)} /></Field>
          <Field label="民族"><Input value={form.ethnicity ?? ""} onChange={(event) => setText("ethnicity", event.target.value)} /></Field>
          <Field label="学历"><Input value={form.education ?? ""} onChange={(event) => setText("education", event.target.value)} /></Field>
          <Field label="学位"><Input value={form.degree ?? ""} onChange={(event) => setText("degree", event.target.value)} /></Field>
        </DialogBody>
        <DialogFooter>
          <Button variant="outline" onClick={onClose}>取消</Button>
          <Button type="submit" disabled={busy}>{busy ? "保存中…" : user ? "保存修改" : "创建用户"}</Button>
        </DialogFooter>
      </form>
    </FormDialog>
  );
}

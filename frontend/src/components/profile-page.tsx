"use client";

import { useEffect, useState } from "react";
import { useRouter } from "next/navigation";
import { useSyncExternalStore } from "react";

import { Icon } from "@/components/icon";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import { WorkspaceShell } from "@/components/workspace-shell";
import { AdminPanel, Field, messageOf } from "@/components/admin-ui";
import { readActiveLogin, roleHomes, type LoginRole } from "@/lib/login";
import { fetchProfile, profileInput, updateProfile, type ProfileInput } from "@/lib/profile";

const emptyInput: ProfileInput = {
  gender: "",
  ethnicity: "",
  birth_date: "",
  admin_title: "",
  professional_title: "",
  research_expertise: "",
  education: "",
  degree: "",
  phone: "",
  email: "",
};

export function ProfileEditor() {
  const [identity, setIdentity] = useState({ name: "", username: "", employeeNo: "", department: "" });
  const [form, setForm] = useState<ProfileInput>(emptyInput);
  const [notice, setNotice] = useState("");
  const [error, setError] = useState("");
  const [saving, setSaving] = useState(false);

  useEffect(() => {
    let cancelled = false;
    async function load() {
      try {
        const profile = await fetchProfile();
        if (cancelled) return;
        setIdentity({
          name: profile.real_name,
          username: profile.username,
          employeeNo: profile.employee_no ?? "未填写",
          department: profile.department_name ?? "未分配部门",
        });
        setForm(profileInput(profile));
      } catch (loadError) {
        if (!cancelled) setError(messageOf(loadError));
      }
    }
    void load();
    return () => {
      cancelled = true;
    };
  }, []);

  function update(field: keyof ProfileInput, value: string) {
    setForm((current) => ({ ...current, [field]: value }));
  }

  async function save() {
    setSaving(true);
    setNotice("");
    setError("");
    try {
      const saved = await updateProfile(form);
      setForm(profileInput(saved));
      setNotice("已保存。之后新建的申报书会带出这些负责人信息，已有申报书不会改。");
    } catch (saveError) {
      setError(messageOf(saveError));
    } finally {
      setSaving(false);
    }
  }

  const genderValue = form.gender === "男" || form.gender === "女" ? form.gender : form.gender ? "other" : "none";

  return (
      <div className="mx-auto max-w-3xl">
        <h1 className="text-[20px] font-semibold text-ink-900">个人资料</h1>
        <p className="mt-1 text-[12.5px] text-ink-500">这些内容作为新建申报书时的负责人信息。姓名、账号和部门由管理员维护。</p>
        <AdminPanel className="mt-5 p-5">
          <div className="grid gap-4 sm:grid-cols-2">
            <ReadOnly label="姓名" value={identity.name || "—"} />
            <ReadOnly label="登录账号" value={identity.username || "—"} />
            <ReadOnly label="工号" value={identity.employeeNo || "—"} />
            <ReadOnly label="所在部门" value={identity.department || "—"} />
            <Field label="性别">
              <Select
                value={genderValue}
                onValueChange={(value) => {
                  if (value === "none") update("gender", "");
                  else if (value !== "other") update("gender", value);
                }}
              >
                <SelectTrigger>
                  <SelectValue placeholder="请选择" />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="none">未填写</SelectItem>
                  <SelectItem value="男">男</SelectItem>
                  <SelectItem value="女">女</SelectItem>
                  {genderValue === "other" ? <SelectItem value="other">{form.gender}</SelectItem> : null}
                </SelectContent>
              </Select>
            </Field>
            <Field label="民族">
              <Input value={form.ethnicity} maxLength={32} onChange={(event) => update("ethnicity", event.target.value)} />
            </Field>
            <Field label="出生年月">
              <Input type="month" value={form.birth_date} onChange={(event) => update("birth_date", event.target.value)} />
            </Field>
            <Field label="手机">
              <Input value={form.phone} maxLength={20} onChange={(event) => update("phone", event.target.value)} />
            </Field>
            <Field label="行政职务">
              <Input value={form.admin_title} maxLength={64} onChange={(event) => update("admin_title", event.target.value)} />
            </Field>
            <Field label="专业技术职务">
              <Input value={form.professional_title} maxLength={64} onChange={(event) => update("professional_title", event.target.value)} />
            </Field>
            <Field label="最后学历">
              <Input value={form.education} maxLength={32} onChange={(event) => update("education", event.target.value)} />
            </Field>
            <Field label="最后学位">
              <Input value={form.degree} maxLength={32} onChange={(event) => update("degree", event.target.value)} />
            </Field>
            <Field label="邮箱" className="sm:col-span-2">
              <Input type="email" value={form.email} maxLength={120} onChange={(event) => update("email", event.target.value)} />
            </Field>
            <Field label="研究专长" className="sm:col-span-2">
              <Input value={form.research_expertise} maxLength={120} onChange={(event) => update("research_expertise", event.target.value)} />
            </Field>
          </div>
          {error ? <p className="mt-4 text-[12.5px] text-cinnabar">{error}</p> : null}
          {notice ? <p className="mt-4 text-[12.5px] text-teal-700">{notice}</p> : null}
          <div className="mt-5 flex justify-end">
            <Button onClick={() => void save()} disabled={saving}>
              <Icon name="mdi:content-save-outline" />
              {saving ? "保存中" : "保存资料"}
            </Button>
          </div>
        </AdminPanel>
      </div>
  );
}

export function ProfilePage() {
  const router = useRouter();
  const role = useSyncExternalStore(subscribeLogin, readShellRole, () => "teacher" as const);

  return (
    <WorkspaceShell
      role={role}
      section="profile"
      onSection={(id) => router.push(id === "stats" ? "/stats" : roleHomes[role])}
      subtitle="个人资料"
    >
      <ProfileEditor />
    </WorkspaceShell>
  );
}

function ReadOnly({ label, value }: { label: string; value: string }) {
  return (
    <div>
      <p className="mb-1 text-[12px] font-medium text-ink-700">{label}</p>
      <p className="flex h-8 items-center rounded-lg bg-ink-50 px-2.5 text-[12.5px] text-ink-800">{value}</p>
    </div>
  );
}

function subscribeLogin() {
  return () => undefined;
}

function readShellRole(): LoginRole {
  return readActiveLogin()?.role ?? "teacher";
}

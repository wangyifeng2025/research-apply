"use client";

import { useRouter } from "next/navigation";
import { useEffect, useState } from "react";

import { AdminDepartments } from "@/components/admin-departments";
import { AdminExperts } from "@/components/admin-experts";
import { AdminFlow } from "@/components/admin-flow";
import { AdminHeader } from "@/components/admin-header";
import { AdminRoles } from "@/components/admin-roles";
import { AdminSidebar } from "@/components/admin-sidebar";
import { AdminStats } from "@/components/admin-stats";
import { AdminTemplates } from "@/components/admin-templates";
import { AdminUsers } from "@/components/admin-users";
import { Icon } from "@/components/icon";
import { Button } from "@/components/ui/button";
import { listCycles, listDepartments, listUsers, type AdminUser, type Cycle, type Department } from "@/lib/admin";
import { apiSend, type AuthUser, type Meta } from "@/lib/api";
import { loginStorageKey } from "@/lib/login";

const metaText: Record<string, { title: string; description: string }> = {
  stats: { title: "数据概览", description: "部门、用户和批次的当前数量" },
  departments: { title: "部门管理", description: "维护扁平部门，供用户归属使用" },
  users: { title: "用户管理", description: "维护账号、部门和角色，可以停用、重置密码" },
  roles: { title: "角色", description: "查看系统内置角色和当前账号数" },
  templates: { title: "申报类别", description: "查看系统支持的申报类别和当前批次" },
  experts: { title: "评审专家", description: "维护带专家角色的账号" },
  flow: { title: "批次与流程", description: "查看申报环节，维护批次开放时间" },
};

export function AdminWorkspace() {
  const router = useRouter();
  const [section, setSection] = useState("departments");
  const [departments, setDepartments] = useState<Department[]>([]);
  const [users, setUsers] = useState<AdminUser[]>([]);
  const [cycles, setCycles] = useState<Cycle[]>([]);
  const [profile, setProfile] = useState<AuthUser | null>(null);
  const [systemMeta, setSystemMeta] = useState<Meta | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState("");
  const [toast, setToast] = useState("");
  const [refreshing, setRefreshing] = useState(false);
  const current = metaText[section] ?? metaText.departments;

  useEffect(() => {
    let cancelled = false;
    async function load() {
      try {
        const [departmentItems, userItems, cycleItems, me, info] = await Promise.all([
          listDepartments(),
          listUsers(),
          listCycles(),
          apiSend<{ user: AuthUser }>("/api/auth/me").catch(() => null),
          apiSend<Meta>("/api/meta").catch(() => null),
        ]);
        if (!cancelled) {
          setDepartments(departmentItems);
          setUsers(userItems);
          setCycles(cycleItems);
          setProfile(me?.user ?? null);
          setSystemMeta(info);
        }
      } catch (reason) {
        if (!cancelled) setError(reason instanceof Error ? reason.message : "管理数据加载失败");
      } finally {
        if (!cancelled) setLoading(false);
      }
    }
    void load();
    return () => { cancelled = true; };
  }, []);

  function notify(message: string) {
    setToast(message);
    window.setTimeout(() => setToast(""), 2800);
  }

  async function refreshAll() {
    const [departmentItems, userItems, cycleItems, info] = await Promise.all([
      listDepartments(),
      listUsers(),
      listCycles(),
      apiSend<Meta>("/api/meta").catch(() => null),
    ]);
    setDepartments(departmentItems);
    setUsers(userItems);
    setCycles(cycleItems);
    setSystemMeta(info);
  }

  async function refresh() {
    setRefreshing(true);
    try {
      await refreshAll();
      notify("数据已刷新");
    } catch (reason) {
      notify(reason instanceof Error ? reason.message : "刷新失败");
    } finally {
      setRefreshing(false);
    }
  }

  async function logout() {
    await fetch("/api/auth/logout", { method: "POST", credentials: "include" }).catch(() => undefined);
    sessionStorage.removeItem(loginStorageKey);
    localStorage.removeItem(loginStorageKey);
    router.push("/");
  }

  return (
    <div className="flex h-dvh flex-col bg-ink-50 font-sans text-ink-800 antialiased">
      <AdminHeader profile={profile} onLogout={() => void logout()} />
      <div className="flex min-h-0 flex-1 flex-col lg:flex-row">
        <AdminSidebar active={section} departmentCount={departments.length} userCount={users.length} onSelect={setSection} onLogout={() => void logout()} />
        <main className="min-w-0 flex-1 overflow-y-auto">
          <div className="mx-auto max-w-[1440px] px-4 py-5 sm:px-6 lg:px-7">
            <div className="flex flex-col justify-between gap-3 sm:flex-row sm:items-end">
              <div>
                <div className="flex items-center gap-1.5 text-[12px] text-ink-400">
                  <span>系统管理</span>
                  <Icon name="mdi:chevron-right" />
                  <span className="text-ink-600">{current.title}</span>
                </div>
                <h1 className="mt-1.5 text-[22px] font-semibold tracking-tight text-ink-900">{current.title}</h1>
                <p className="mt-1 text-[13px] text-ink-500">{current.description}</p>
              </div>
              <Button variant="outline" disabled={refreshing || loading} onClick={() => void refresh()}>
                <Icon name="mdi:refresh" />{refreshing ? "正在刷新…" : "刷新"}
              </Button>
            </div>
            {loading ? <div className="mt-4 rounded-xl border border-ink-200 bg-white px-5 py-16 text-center text-[13px] text-ink-400">正在加载管理数据…</div> : null}
            {!loading && error ? <div className="mt-4 rounded-xl border border-red-200 bg-red-50 px-5 py-6 text-[13px] text-red-700">{error}。请用管理员账号登录后再打开此页。</div> : null}
            {!loading && !error && section === "stats" ? <AdminStats users={users} departments={departments} cycles={cycles} /> : null}
            {!loading && !error && section === "departments" ? <AdminDepartments departments={departments} onChanged={refreshAll} onNotify={notify} /> : null}
            {!loading && !error && section === "users" ? <AdminUsers users={users} departments={departments} onChanged={refreshAll} onNotify={notify} /> : null}
            {!loading && !error && section === "roles" ? <AdminRoles users={users} /> : null}
            {!loading && !error && section === "templates" ? <AdminTemplates meta={systemMeta} /> : null}
            {!loading && !error && section === "experts" ? <AdminExperts users={users} departments={departments} onChanged={refreshAll} onNotify={notify} /> : null}
            {!loading && !error && section === "flow" ? <AdminFlow cycles={cycles} workflow={systemMeta?.workflow ?? []} onChanged={refreshAll} onNotify={notify} /> : null}
          </div>
        </main>
      </div>
      {toast ? <p role="status" className="fixed bottom-6 left-1/2 z-[80] max-w-[calc(100vw-2rem)] -translate-x-1/2 rounded-lg bg-ink-900 px-4 py-2.5 text-center text-[13px] text-white shadow-lg">{toast}</p> : null}
    </div>
  );
}

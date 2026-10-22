"use client";

import { useRouter } from "next/navigation";
import { useEffect, useState, type ReactNode } from "react";

import { Icon } from "@/components/icon";
import {
  loginStorageKey,
  readActiveLogin,
  roleNav,
  rolePresets,
  type LoginRole,
} from "@/lib/login";

export function WorkspaceShell({
  role,
  section,
  onSection,
  subtitle,
  sidebar,
  children,
}: {
  role: LoginRole;
  section: string;
  onSection: (id: string) => void;
  subtitle: string;
  sidebar?: ReactNode;
  children: ReactNode;
}) {
  const router = useRouter();
  const [name, setName] = useState(rolePresets[role].hint.split(" · ")[0] ?? "");
  const [detail, setDetail] = useState(rolePresets[role].hint);
  const [menuOpen, setMenuOpen] = useState(false);
  const nav = roleNav[role];

  useEffect(() => {
    const stored = readActiveLogin();
    if (!stored || stored.role !== role) {
      return;
    }
    const displayName = stored.realName ?? rolePresets[role].hint.split(" · ")[0] ?? stored.username;
    setName(displayName);
    setDetail(
      stored.departmentName
        ? `${rolePresets[role].label} · ${stored.departmentName}`
        : rolePresets[role].hint,
    );
  }, [role]);

  async function logout() {
    await fetch("/api/auth/logout", { method: "POST", credentials: "include" }).catch(() => undefined);
    sessionStorage.removeItem(loginStorageKey);
    localStorage.removeItem(loginStorageKey);
    router.push("/");
  }

  return (
    <div className="flex h-dvh flex-col bg-ink-50 text-ink-800 antialiased">
      <header className="z-40 flex h-16 shrink-0 items-center gap-4 border-b border-ink-200/80 bg-white px-4 lg:px-5">
        <div className="flex shrink-0 items-center gap-2.5">
          <div className="flex h-9 w-9 items-center justify-center rounded-lg bg-gradient-to-br from-teal-500 to-ink-800 text-sm font-semibold text-white shadow-sm">
            研
          </div>
          <div className="leading-tight">
            <div className="text-[14.5px] font-semibold tracking-wide text-ink-900">科研项目管理平台</div>
            <div className="mt-0.5 hidden text-[10.5px] text-ink-400 sm:block">{subtitle}</div>
          </div>
        </div>
        <nav className="ml-2 hidden items-center gap-0.5 text-[13px] md:flex">
          {nav.map((item) => {
            const active = item.id === section;
            return (
              <button
                key={item.id}
                type="button"
                onClick={() => onSection(item.id)}
                className={`rounded-md px-3 py-2 transition ${
                  active
                    ? "bg-teal-50 font-medium text-teal-700"
                    : "text-ink-600 hover:bg-ink-50 hover:text-ink-900"
                }`}
              >
                {item.label}
              </button>
            );
          })}
        </nav>
        <div className="relative ml-auto">
          <button
            type="button"
            className="flex items-center gap-2 rounded-lg py-1 pr-2 pl-1 hover:bg-ink-50"
            onClick={() => setMenuOpen((open) => !open)}
          >
            <span className="flex h-8 w-8 items-center justify-center rounded-full bg-teal-100 text-[12px] font-semibold text-teal-800">
              {name.slice(0, 1)}
            </span>
            <span className="hidden text-left sm:block">
              <span className="block text-[12px] leading-4 font-medium text-ink-800">{name}</span>
              <span className="block text-[10.5px] leading-4 text-ink-400">{detail}</span>
            </span>
          </button>
          {menuOpen ? (
            <div className="absolute right-0 mt-2 w-44 rounded-xl border border-ink-200 bg-white p-1.5 shadow-xl">
              <button
                type="button"
                className="flex w-full items-center gap-2 rounded-lg px-2.5 py-2 text-left text-[12px] text-red-600 hover:bg-red-50"
                onClick={() => void logout()}
              >
                <Icon name="mdi:logout" className="text-base" />
                退出登录
              </button>
            </div>
          ) : null}
        </div>
      </header>
      <div className="flex min-h-0 flex-1">
        {sidebar}
        <main className="min-w-0 flex-1 overflow-y-auto">
          <div className="mx-auto max-w-[1440px] px-4 py-5 sm:px-6 lg:px-7">{children}</div>
        </main>
      </div>
      <nav className="flex shrink-0 gap-1 border-t border-ink-200 bg-white p-2 md:hidden">
        {nav.map((item) => (
          <button
            key={item.id}
            type="button"
            onClick={() => onSection(item.id)}
            className={`flex-1 rounded-md px-2 py-2 text-[12px] ${
              item.id === section ? "bg-teal-50 font-medium text-teal-700" : "text-ink-600"
            }`}
          >
            {item.label}
          </button>
        ))}
      </nav>
    </div>
  );
}

export function SideNav({
  title,
  items,
  active,
  onSelect,
  onLogout,
  footer,
}: {
  title: string;
  items: { id: string; label: string; icon: string; badge?: string; gold?: boolean; divider?: boolean }[];
  active: string;
  onSelect: (id: string) => void;
  onLogout?: () => void;
  footer?: ReactNode;
}) {
  const router = useRouter();

  async function logout() {
    if (onLogout) {
      onLogout();
      return;
    }
    await fetch("/api/auth/logout", { method: "POST", credentials: "include" }).catch(() => undefined);
    sessionStorage.removeItem(loginStorageKey);
    localStorage.removeItem(loginStorageKey);
    router.push("/");
  }

  return (
    <aside className="hidden w-[228px] shrink-0 flex-col overflow-y-auto border-r border-ink-200/80 bg-white lg:flex">
      <p className="px-4 pt-5 pb-2 text-[10.5px] font-semibold tracking-[0.16em] text-ink-400">{title}</p>
      <nav className="flex-1 space-y-0.5 px-2.5 pb-4 text-[13px]">
        {items.map((item) => {
          const selected = item.id === active;
          return (
            <div key={item.id}>
            {item.divider ? <div className="my-2 border-t border-ink-100" /> : null}
            <button
              type="button"
              onClick={() => onSelect(item.id)}
              className={`flex w-full items-center gap-2.5 rounded-lg px-3 py-2.5 text-left transition ${
                selected ? "bg-teal-50 font-medium text-teal-700" : "text-ink-600 hover:bg-ink-50 hover:text-ink-900"
              }`}
            >
              <Icon name={item.icon} className="text-lg" />
              <span className="min-w-0 flex-1 truncate">{item.label}</span>
              {item.badge ? (
                <span className={`rounded px-1.5 py-0.5 text-[10px] ${item.gold ? "bg-gold-500 font-semibold text-white" : selected ? "bg-teal-100 text-teal-700" : "bg-ink-100 text-ink-500"}`}>
                  {item.badge}
                </span>
              ) : null}
            </button>
            </div>
          );
        })}
      </nav>
      <div className="border-t border-ink-100 p-3">
        {footer}
        <button
          type="button"
          onClick={() => void logout()}
          className="flex w-full items-center gap-2 rounded-lg px-3 py-2 text-[12.5px] text-ink-500 hover:bg-ink-50 hover:text-ink-800"
        >
          <Icon name="mdi:logout" className="text-base" />
          退出登录
        </button>
      </div>
    </aside>
  );
}

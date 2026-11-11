"use client";

import Link from "next/link";
import { useRouter } from "next/navigation";
import { useSyncExternalStore, type ReactNode } from "react";

import { Icon } from "@/components/icon";
import { Button } from "@/components/ui/button";
import { DropdownMenu, DropdownMenuContent, DropdownMenuItem, DropdownMenuTrigger } from "@/components/ui/dropdown-menu";
import {
  loginStorageKey,
  readActiveLogin,
  roleNav,
  rolePresets,
  type LoginRole,
} from "@/lib/login";
import { cn } from "@/lib/utils";

export function WorkspaceShell({
  role,
  section,
  onSection,
  subtitle,
  sidebar,
  onProfile,
  children,
}: {
  role: LoginRole;
  section: string;
  onSection: (id: string) => void;
  subtitle: string;
  sidebar?: ReactNode;
  onProfile?: () => void;
  children: ReactNode;
}) {
  const router = useRouter();
  const profile = useSyncExternalStore(
    subscribeLogin,
    () => readShellProfile(role),
    () => fallbackShellProfile(role),
  );
  const [name, detail] = profile.split("\u0000");
  const nav = roleNav[role];

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
          <div className="flex h-9 w-9 items-center justify-center rounded-lg bg-linear-to-br from-teal-500 to-ink-800 text-sm font-semibold text-white shadow-sm">
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
            const className = cn("h-auto px-3 py-2 text-[13px] font-normal", active ? "bg-teal-50 font-medium text-teal-700" : "text-ink-600");
            if (item.href) {
              return (
                <Button key={item.id} variant="ghost" asChild className={className}>
                  <Link href={item.href}>{item.label}</Link>
                </Button>
              );
            }
            return (
              <Button key={item.id} variant="ghost" onClick={() => onSection(item.id)} className={className}>
                {item.label}
              </Button>
            );
          })}
        </nav>
        <div className="ml-auto">
          <DropdownMenu>
            <DropdownMenuTrigger asChild>
              <Button variant="ghost" className="h-auto px-1.5 py-1">
                <span className="flex h-8 w-8 items-center justify-center rounded-full bg-teal-100 text-[12px] font-semibold text-teal-800">
                  {name.slice(0, 1)}
                </span>
                <span className="hidden text-left sm:block">
                  <span className="block text-[12px] leading-4 font-medium text-ink-800">{name}</span>
                  <span className="block text-[10.5px] leading-4 text-ink-400">{detail}</span>
                </span>
                <Icon name="mdi:chevron-down" className="text-ink-400" />
              </Button>
            </DropdownMenuTrigger>
            <DropdownMenuContent align="end" className="w-44">
              <DropdownMenuItem onSelect={() => (onProfile ? onProfile() : router.push("/profile"))}>
                <Icon name="mdi:account-outline" className="text-base" />
                个人资料
              </DropdownMenuItem>
              <DropdownMenuItem className="text-red-600 data-highlighted:bg-red-50" onSelect={() => void logout()}>
                <Icon name="mdi:logout" className="text-base" />
                退出登录
              </DropdownMenuItem>
            </DropdownMenuContent>
          </DropdownMenu>
        </div>
      </header>
      <div className="flex min-h-0 flex-1">
        {sidebar}
        <main className="min-w-0 flex-1 overflow-y-auto">
          <div className="mx-auto max-w-360 px-4 py-5 sm:px-6 lg:px-7">{children}</div>
        </main>
      </div>
      <nav className="flex shrink-0 gap-1 border-t border-ink-200 bg-white p-2 md:hidden">
        {nav.map((item) => {
          const className = cn("h-auto flex-1 px-2 py-2 text-[12px] font-normal", item.id === section ? "bg-teal-50 font-medium text-teal-700" : "text-ink-600");
          if (item.href) {
            return (
              <Button key={item.id} variant="ghost" asChild className={className}>
                <Link href={item.href}>{item.label}</Link>
              </Button>
            );
          }
          return (
            <Button key={item.id} variant="ghost" onClick={() => onSection(item.id)} className={className}>
              {item.label}
            </Button>
          );
        })}
      </nav>
    </div>
  );
}

function subscribeLogin() {
  return () => undefined;
}

function fallbackShellProfile(role: LoginRole) {
  const label = rolePresets[role].label;
  return `${label}\u0000${label}`;
}

function readShellProfile(role: LoginRole) {
  const stored = readActiveLogin();
  if (!stored || stored.role !== role) return fallbackShellProfile(role);
  const name = stored.realName ?? stored.username;
  const detail = stored.departmentName
    ? `${rolePresets[role].label} · ${stored.departmentName}`
    : rolePresets[role].label;
  return `${name}\u0000${detail}`;
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
    <aside className="hidden w-57 shrink-0 flex-col overflow-y-auto border-r border-ink-200/80 bg-white lg:flex">
      <p className="px-4 pt-5 pb-2 text-[10.5px] font-semibold tracking-[0.16em] text-ink-400">{title}</p>
      <nav className="flex-1 space-y-0.5 px-2.5 pb-4 text-[13px]">
        {items.map((item) => {
          const selected = item.id === active;
          return (
            <div key={item.id}>
            {item.divider ? <div className="my-2 border-t border-ink-100" /> : null}
            <Button
              type="button"
              variant="ghost"
              onClick={() => onSelect(item.id)}
              className={cn("h-auto w-full justify-start gap-2.5 px-3 py-2.5 text-left text-[13px] font-normal", selected ? "bg-teal-50 font-medium text-teal-700" : "text-ink-600")}
            >
              <Icon name={item.icon} className="text-lg" />
              <span className="min-w-0 flex-1 truncate">{item.label}</span>
              {item.badge ? (
                <span className={`rounded px-1.5 py-0.5 text-[10px] ${item.gold ? "bg-gold-500 font-semibold text-white" : selected ? "bg-teal-100 text-teal-700" : "bg-ink-100 text-ink-500"}`}>
                  {item.badge}
                </span>
              ) : null}
            </Button>
            </div>
          );
        })}
      </nav>
      <div className="border-t border-ink-100 p-3">
        {footer}
        <Button variant="ghost" onClick={() => void logout()} className="h-auto w-full justify-start gap-2 px-3 py-2 text-[12.5px] font-normal text-ink-500">
          <Icon name="mdi:logout" className="text-base" />
          退出登录
        </Button>
      </div>
    </aside>
  );
}

"use client";

import Link from "next/link";
import { useRouter } from "next/navigation";

import { Icon } from "@/components/icon";
import { Button } from "@/components/ui/button";
import { DropdownMenu, DropdownMenuContent, DropdownMenuItem, DropdownMenuTrigger } from "@/components/ui/dropdown-menu";
import { roleLabel } from "@/lib/admin";
import type { AuthUser } from "@/lib/api";

export function AdminHeader({ profile, onLogout }: { profile: AuthUser | null; onLogout: () => void }) {
  const name = profile?.real_name || "管理员";
  const department = profile?.department_name || "未分配部门";
  const roles = profile?.roles.map((role) => roleLabel(role)).join("、") || "系统管理员";
  const router = useRouter();

  return (
    <header className="relative z-40 flex h-16 shrink-0 items-center gap-4 border-b border-ink-200/80 bg-white px-4 lg:px-5">
      <Link href="/admin" className="flex shrink-0 items-center gap-2.5">
        <span className="flex h-9 w-9 items-center justify-center rounded-lg bg-teal-700 text-white"><Icon name="mdi:flask-outline" className="text-xl" /></span>
        <span className="leading-tight">
          <span className="block text-[15px] font-semibold text-ink-900">科研项目管理平台</span>
          <span className="mt-0.5 hidden text-[11px] text-ink-400 sm:block">管理控制台</span>
        </span>
      </Link>
      <nav className="ml-2 hidden items-center md:flex">
        <Link href="/admin" className="rounded-md bg-teal-50 px-3 py-2 text-[13px] font-medium text-teal-700">系统管理</Link>
        <Link href="/stats" className="rounded-md px-3 py-2 text-[13px] text-ink-600">统计</Link>
      </nav>
      <div className="ml-auto">
        <DropdownMenu>
          <DropdownMenuTrigger asChild>
            <Button variant="ghost" className="h-auto px-1.5 py-1">
              <span className="flex h-8 w-8 items-center justify-center rounded-full bg-ink-900 text-[12px] font-semibold text-white">{name.slice(0, 1)}</span>
              <span className="hidden text-left sm:block">
                <span className="block text-[12px] leading-4 font-medium text-ink-800">{name}</span>
                <span className="block text-[11px] leading-4 text-ink-400">{roles} · {department}</span>
              </span>
              <Icon name="mdi:chevron-down" className="text-ink-400" />
            </Button>
          </DropdownMenuTrigger>
          <DropdownMenuContent align="end" className="w-44">
            <DropdownMenuItem onSelect={() => router.push("/profile")}>
              <Icon name="mdi:account-outline" />个人资料
            </DropdownMenuItem>
            <DropdownMenuItem className="text-red-600 data-highlighted:bg-red-50" onSelect={onLogout}>
              <Icon name="mdi:logout" />退出登录
            </DropdownMenuItem>
          </DropdownMenuContent>
        </DropdownMenu>
      </div>
    </header>
  );
}

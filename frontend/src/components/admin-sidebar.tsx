"use client";

import { Icon } from "@/components/icon";
import { Button } from "@/components/ui/button";
import { cn } from "@/lib/utils";

const items = [
  { id: "stats", label: "数据概览", icon: "mdi:chart-box-outline" },
  { id: "departments", label: "部门管理", icon: "mdi:office-building-cog-outline" },
  { id: "users", label: "用户管理", icon: "mdi:account-multiple-outline" },
  { id: "roles", label: "角色", icon: "mdi:shield-account-outline" },
  { id: "templates", label: "申报类别", icon: "mdi:file-document-edit-outline" },
  { id: "experts", label: "评审专家", icon: "mdi:account-supervisor-outline" },
  { id: "flow", label: "批次与流程", icon: "mdi:sitemap-outline" },
];

export function AdminSidebar({ active, departmentCount, userCount, onSelect, onLogout }: { active: string; departmentCount: number; userCount: number; onSelect: (id: string) => void; onLogout: () => void }) {
  return (
    <>
      <aside className="hidden w-57 shrink-0 flex-col border-r border-ink-200/80 bg-white lg:flex">
        <p className="px-4 pt-5 pb-2 text-[12px] font-medium text-ink-400">系统管理</p>
        <nav className="flex-1 space-y-0.5 overflow-y-auto px-2.5 pb-4 text-[13px]">
          {items.map((item) => {
            const badge = item.id === "departments" ? departmentCount : item.id === "users" ? userCount : undefined;
            return (
              <Button key={item.id} variant="ghost" onClick={() => onSelect(item.id)} className={cn("h-auto w-full justify-start gap-2.5 px-3 py-2.5 text-left text-[13px] font-normal", active === item.id ? "bg-teal-50 font-medium text-teal-700" : "text-ink-600")}>
                <Icon name={item.icon} className="text-[18px]" />
                <span>{item.label}</span>
                {badge !== undefined ? <span className="ml-auto rounded bg-ink-100 px-1.5 py-0.5 text-[11px] text-ink-500">{badge.toLocaleString()}</span> : null}
              </Button>
            );
          })}
        </nav>
        <div className="border-t border-ink-100 p-3">
          <Button variant="ghost" onClick={onLogout} className="h-auto w-full justify-start gap-2.5 px-3 py-2.5 text-left text-[13px] font-normal text-ink-600">
            <Icon name="mdi:logout" className="text-[18px]" />退出登录
          </Button>
        </div>
      </aside>
      <nav className="flex shrink-0 gap-1 overflow-x-auto border-b border-ink-200 bg-white px-3 py-2 lg:hidden">
        {items.map((item) => <Button key={item.id} variant="ghost" size="sm" onClick={() => onSelect(item.id)} className={cn("shrink-0", active === item.id ? "bg-teal-50 text-teal-700" : "text-ink-600")}>{item.label}</Button>)}
        <Button variant="ghost" size="sm" onClick={onLogout} className="shrink-0 text-red-600">退出</Button>
      </nav>
    </>
  );
}

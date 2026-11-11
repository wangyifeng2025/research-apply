import type { ReactNode } from "react";

import { Icon } from "@/components/icon";
import { cn } from "@/lib/utils";

export function AdminStat({ label, value, unit, icon, foot, gold = false }: { label: string; value: string | number; unit?: string; icon?: string; foot?: ReactNode; gold?: boolean }) {
  return (
    <section className="rounded-xl border border-ink-200/80 bg-white p-4 shadow-sm">
      <div className="flex items-start justify-between">
        <div>
          <p className="text-[12px] text-ink-500">{label}</p>
          <p className={cn("mt-2 text-[22px] leading-none font-semibold", gold ? "text-gold-700" : "text-ink-900")}>
            {typeof value === "number" ? value.toLocaleString() : value}
            {unit ? <span className="ml-1 text-[11px] font-normal text-ink-400">{unit}</span> : null}
          </p>
        </div>
        {icon ? <span className={cn("flex h-9 w-9 items-center justify-center rounded-lg", gold ? "bg-gold-50 text-gold-600" : "bg-teal-50 text-teal-600")}><Icon name={icon} /></span> : null}
      </div>
      {foot ? <div className="mt-3 text-[11px] text-ink-400">{foot}</div> : null}
    </section>
  );
}

export function AdminPanel({ children, className = "" }: { children: ReactNode; className?: string }) {
  return <section className={cn("rounded-xl border border-ink-200/80 bg-white shadow-sm", className)}>{children}</section>;
}

export function Field({ label, required = false, children, className }: { label: string; required?: boolean; children: ReactNode; className?: string }) {
  return (
    <label className={cn("block", className)}>
      <span className="mb-1 block text-[12px] font-medium text-ink-700">
        {label}
        {required ? <span className="ml-1 text-cinnabar">*</span> : null}
      </span>
      {children}
    </label>
  );
}

export function messageOf(error: unknown): string {
  return error instanceof Error ? error.message : "操作失败，请重试";
}

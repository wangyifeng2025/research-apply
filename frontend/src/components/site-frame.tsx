import type { ReactNode } from "react";
import Link from "next/link";

const links = [
  { href: "/", label: "首页" },
  { href: "/applications", label: "申报" },
  { href: "/reviews/department", label: "系审核" },
  { href: "/reviews/office", label: "科研处" },
  { href: "/reviews/expert", label: "专家盲审" },
];

export function SiteFrame({ children }: { children: ReactNode }) {
  return (
    <div className="relative mx-auto flex min-h-screen w-full max-w-6xl flex-col px-5 py-6 sm:px-8">
      <header className="flex flex-wrap items-end justify-between gap-6 border-b border-[var(--color-line)] pb-5">
        <div>
          <p className="text-xs tracking-[0.28em] text-[var(--color-ink-soft)]">
            黔西南民族职业技术学院 · 科研处
          </p>
          <Link href="/" className="mt-2 block font-serif text-3xl tracking-wide">
            院级科研项目申报
          </Link>
        </div>
        <nav className="flex flex-wrap gap-x-5 gap-y-2 text-sm">
          {links.map((link) => (
            <Link
              key={link.href}
              href={link.href}
              className="border-b border-transparent pb-1 hover:border-[var(--color-cinnabar)]"
            >
              {link.label}
            </Link>
          ))}
        </nav>
      </header>
      <main className="flex-1 py-8">{children}</main>
    </div>
  );
}

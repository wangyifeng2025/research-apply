import type { Metadata } from "next";
import Link from "next/link";

import { buttonVariants } from "@/components/ui/button";
import { cn } from "@/lib/utils";

export const metadata: Metadata = {
  title: "没有访问权限 · 院级科研项目申报",
};

export default function ForbiddenPage() {
  return (
    <main className="flex min-h-dvh items-center bg-ink-50 px-6 py-16 text-ink-800">
      <div className="mx-auto w-full max-w-md">
        <div className="flex items-center gap-2.5">
          <div className="flex h-9 w-9 items-center justify-center rounded-lg bg-linear-to-br from-teal-500 to-ink-800 text-sm font-semibold text-white">
            研
          </div>
          <p className="text-[13px] text-ink-500">院级科研项目申报</p>
        </div>
        <h1 className="mt-8 text-[28px] leading-tight font-semibold tracking-tight text-ink-900">没有访问这个页面的权限</h1>
        <p className="mt-3 text-[13px] leading-6 text-ink-600">
          当前账号的角色打不开这里。请回到自己的工作台；如果还没有登录，请先登录后再进入对应栏目。
        </p>
        <Link href="/" className={cn(buttonVariants({ size: "lg" }), "mt-6")}>
          返回登录
        </Link>
      </div>
    </main>
  );
}

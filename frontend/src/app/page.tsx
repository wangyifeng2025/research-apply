import type { Metadata } from "next";

import { LoginForm } from "@/components/login-form";
import { cycleStatusLabel, fetchHealth, fetchMeta } from "@/lib/api";

export const dynamic = "force-dynamic";

export const metadata: Metadata = {
  title: "登录 · 院级科研项目申报",
};

export default async function RootPage() {
  const [health, meta] = await Promise.all([fetchHealth(), fetchMeta()]);

  return (
    <LoginForm
      cycleTitle={meta?.cycle?.title ?? "2026年院级科研项目申报"}
      cycleStatus={meta?.cycle ? cycleStatusLabel(meta.cycle.status) : null}
      budgetSubjectCount={meta?.budget_subject_count ?? null}
      database={meta?.database ?? "research_apply_db"}
      connected={health?.database === "up"}
    />
  );
}

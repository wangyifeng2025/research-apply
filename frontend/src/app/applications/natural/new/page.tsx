import { CoverFields } from "@/components/cover-fields";
import { SectionList } from "@/components/section-list";
import { naturalSections } from "@/lib/sections";

export default function NewNaturalApplicationPage() {
  return (
    <div className="grid gap-8">
      <div>
        <p className="text-xs tracking-[0.24em] text-[var(--color-pine)]">附件 3 · 自然科学类</p>
        <h1 className="mt-2 font-serif text-4xl">自然科学类项目申报书</h1>
        <p className="mt-3 text-sm text-[var(--color-ink-soft)]">
          同一套栏目同时用于 A.自然科学类和 D.双高校建设专项。
        </p>
      </div>
      <CoverFields category="A.自然科学类 / D.双高校建设专项" />
      <SectionList sections={naturalSections} />
    </div>
  );
}

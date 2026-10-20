import { CoverFields } from "@/components/cover-fields";
import { SectionList } from "@/components/section-list";
import { socialSections } from "@/lib/sections";

export default function NewSocialApplicationPage() {
  return (
    <div className="grid gap-8">
      <div>
        <p className="text-xs tracking-[0.24em] text-[var(--color-indigo)]">附件 2 · 社会科学类</p>
        <h1 className="mt-2 font-serif text-4xl">社会科学类项目申报书</h1>
      </div>
      <CoverFields category="B.社会科学类" />
      <SectionList sections={socialSections} />
    </div>
  );
}

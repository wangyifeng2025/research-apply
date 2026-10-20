import type { FormSection } from "@/lib/sections";

export function SectionList({ sections }: { sections: FormSection[] }) {
  return (
    <ol className="grid gap-4">
      {sections.map((section, index) => (
        <li
          key={section.title}
          className="rise grid gap-3 border border-[var(--color-line)] bg-[var(--color-card)] p-5 sm:grid-cols-[4.5rem_1fr]"
          style={{ animationDelay: `${index * 40}ms` }}
        >
          <div className="font-serif text-2xl text-[var(--color-cinnabar)]">{section.index}</div>
          <div>
            <div className="flex flex-wrap items-baseline justify-between gap-2">
              <h2 className="font-serif text-xl">{section.title}</h2>
              <p className="text-xs tracking-wide text-[var(--color-ink-soft)]">{section.table}</p>
            </div>
            <ul className="mt-3 grid gap-1 text-sm leading-6 text-[var(--color-ink-soft)]">
              {section.points.map((point) => (
                <li key={point}>{point}</li>
              ))}
            </ul>
          </div>
        </li>
      ))}
    </ol>
  );
}

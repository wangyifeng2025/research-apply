"use client";

const fields = [
  ["项目名称", "准确、简明，最多 40 个字"],
  ["项目负责人", "与账号姓名一致，提交时写入快照"],
  ["联系电话", "封面联系电话"],
  ["负责人所在部门", "系审核按此部门匹配"],
  ["填表日期", ""],
  ["关键词", "最多 5 个，用分号隔开"],
] as const;

export function CoverFields({ category }: { category: string }) {
  return (
    <form className="border border-[var(--color-ink)] bg-[var(--color-card)] p-5 sm:p-7">
      <div className="flex items-start justify-between gap-4">
        <div>
          <p className="text-xs tracking-[0.22em] text-[var(--color-ink-soft)]">封面</p>
          <h2 className="mt-1 font-serif text-2xl">项目基本情况</h2>
        </div>
        <p className="max-w-40 text-right font-serif text-sm leading-6 text-[var(--color-cinnabar)]">
          {category}
        </p>
      </div>
      <div className="mt-6 grid gap-4 sm:grid-cols-2">
        {fields.map(([label, hint]) => (
          <label key={label} className="grid gap-1 text-sm">
            <span>{label}</span>
            <input
              name={label}
              className="border-b border-[var(--color-line)] bg-transparent px-1 py-2 outline-none focus:border-[var(--color-cinnabar)]"
              placeholder={hint}
            />
          </label>
        ))}
      </div>
    </form>
  );
}

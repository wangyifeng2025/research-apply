import Link from "next/link";

export default function ApplicationsPage() {
  return (
    <div className="grid gap-8">
      <div>
        <p className="text-xs tracking-[0.24em] text-[var(--color-ink-soft)]">教师填报</p>
        <h1 className="mt-2 font-serif text-4xl">我的申报</h1>
        <p className="mt-3 max-w-2xl text-sm leading-7 text-[var(--color-ink-soft)]">
          草稿可反复修改。提交前须确认申请者承诺，并写清项目名称、负责人和所在部门。退回后可再次提交，审核意见会保留。
        </p>
      </div>
      <div className="grid gap-4 sm:grid-cols-2">
        <Link
          href="/applications/social/new"
          className="border border-[var(--color-line)] bg-[var(--color-card)] p-6"
        >
          <p className="text-xs tracking-[0.2em] text-[var(--color-indigo)]">社会科学</p>
          <h2 className="mt-2 font-serif text-2xl">新建社会科学类申报</h2>
        </Link>
        <Link
          href="/applications/natural/new"
          className="border border-[var(--color-line)] bg-[var(--color-card)] p-6"
        >
          <p className="text-xs tracking-[0.2em] text-[var(--color-pine)]">自然科学</p>
          <h2 className="mt-2 font-serif text-2xl">新建自然科学类申报</h2>
        </Link>
      </div>
      <div className="border border-dashed border-[var(--color-line)] px-5 py-10 text-sm text-[var(--color-ink-soft)]">
        申报列表会在教师账号和申报接口接入后显示。
      </div>
    </div>
  );
}

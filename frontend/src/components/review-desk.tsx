import { departmentChecks } from "@/lib/sections";

const copy = {
  department: {
    eyebrow: "第一审",
    title: "系审核",
    body: "审核本部门教师的申报。通过后进入科研处，退回后教师可修改再报。",
  },
  office: {
    eyebrow: "第二审",
    title: "科研处审核",
    body: "复核申报书并分配盲审专家。封面编号在此环节填写。",
  },
  expert: {
    eyebrow: "第三审",
    title: "专家盲审",
    body: "专家只看申报内容。申请人不能看到评审专家身份。",
  },
} as const;

export function ReviewDesk({ stage }: { stage: keyof typeof copy }) {
  const page = copy[stage];

  return (
    <div className="grid gap-8">
      <div>
        <p className="text-xs tracking-[0.24em] text-[var(--color-cinnabar)]">{page.eyebrow}</p>
        <h1 className="mt-2 font-serif text-4xl">{page.title}</h1>
        <p className="mt-3 max-w-2xl text-sm leading-7 text-[var(--color-ink-soft)]">{page.body}</p>
      </div>
      {stage === "department" ? (
        <ul className="grid gap-3">
          {departmentChecks.map((item, index) => (
            <li
              key={item}
              className="border border-[var(--color-line)] bg-[var(--color-card)] px-5 py-4 text-sm leading-7"
            >
              <span className="mr-3 font-serif text-[var(--color-cinnabar)]">0{index + 1}</span>
              {item}
            </li>
          ))}
        </ul>
      ) : (
        <div className="border border-[var(--color-line)] bg-[var(--color-card)] px-5 py-8 text-sm leading-7 text-[var(--color-ink-soft)]">
          待审列表会在对应角色登录后，从 reviews 与 expert_assignments 读取。
        </div>
      )}
    </div>
  );
}

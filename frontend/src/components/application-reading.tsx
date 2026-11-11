import { AttachmentPanel } from "@/components/attachment-panel";
import { ExportPdfButton } from "@/components/export-pdf-button";
import type { ApplicationDetail } from "@/lib/applications";

export function ApplicationReading({ detail }: { detail: ApplicationDetail }) {
  const natural = detail.project_class !== "social";
  return (
    <div className="space-y-4 text-[12.5px] leading-6 text-ink-700">
      <div className="flex justify-end">
        <ExportPdfButton applicationId={detail.id} size="sm" />
      </div>
      <dl className="grid grid-cols-2 gap-x-4 gap-y-1 sm:grid-cols-3">
        <Fact label="负责人" value={detail.principal_name} />
        <Fact label="部门" value={detail.department_name} />
        <Fact label="职称" value={detail.professional_title} />
        <Fact label="学历" value={detail.education} />
        <Fact label="学位" value={detail.degree} />
        <Fact label="手机" value={detail.mobile_phone} />
      </dl>
      {detail.conclusion_opinion ? <Paragraph title="立项结论说明" text={detail.conclusion_opinion} /> : null}
      <Paragraph title="关键词" text={detail.keywords} />
      <section>
        <h3 className="font-medium text-ink-900">主要参加者</h3>
        {detail.members.length === 0 ? (
          <p className="mt-1 text-ink-400">无参加者</p>
        ) : (
          <ul className="mt-1 space-y-1">
            {detail.members.map((member) => (
              <li key={`${member.name}-${member.organization ?? ""}`}>
                {member.name}
                {member.professional_title ? ` · ${member.professional_title}` : ""}
                {member.organization ? ` · ${member.organization}` : ""}
              </li>
            ))}
          </ul>
        )}
      </section>
      <section>
        <h3 className="font-medium text-ink-900">经费合计 {detail.budget.total} 万元</h3>
        <p className="text-ink-500">直接费用 {detail.budget.direct_total}，间接费用 {detail.budget.indirect_total}</p>
        <ul className="mt-1 space-y-0.5">
          {detail.budget.lines.filter((line) => filledAmount(line.amount)).map((line) => (
            <li key={line.code} className="flex justify-between gap-3">
              <span>{line.name}</span>
              <span className="tabular-nums">{line.amount}</span>
            </li>
          ))}
        </ul>
      </section>
      {natural ? <NaturalReading detail={detail} /> : <SocialReading detail={detail} />}
      {detail.recommender ? (
        <section>
          <h3 className="font-medium text-ink-900">推荐人 {detail.recommender.name}</h3>
          <p>
            {detail.recommender.professional_title} · {detail.recommender.organization}
          </p>
          <Paragraph title="推荐意见" text={detail.recommender.opinion} />
        </section>
      ) : null}
      <AttachmentPanel applicationId={detail.id} items={detail.attachments} editable={false} />
    </div>
  );
}

function SocialReading({ detail }: { detail: ApplicationDetail }) {
  return (
    <>
      <Paragraph title="选题依据" text={detail.ss_design.topic_basis} />
      <Paragraph title="研究内容" text={detail.ss_design.research_content} />
      <Paragraph title="研究方法" text={detail.ss_design.methodology} />
      <Paragraph title="创新之处" text={detail.ss_design.innovation} />
      <Paragraph title="预期成果" text={detail.ss_design.expected_outcome} />
      <Paragraph title="参考文献" text={detail.ss_design.reference_list} />
      <Paragraph title="学术简历" text={detail.ss_foundation.academic_resume} />
      <Paragraph title="研究基础" text={detail.ss_foundation.research_basis} />
      <Paragraph title="已承担项目" text={detail.ss_foundation.undertaken_projects} />
      <Paragraph title="保障条件" text={detail.ss_foundation.support_conditions} />
      <RowList title="前期成果" rows={detail.ss_achievements.map((item) => item.title)} />
      <RowList title="阶段成果" rows={detail.ss_stage_outputs.map((item) => item.title)} />
      <RowList title="最终成果" rows={detail.ss_final_outputs.map((item) => item.title)} />
    </>
  );
}

function NaturalReading({ detail }: { detail: ApplicationDetail }) {
  const indicators = indicatorLines(detail.ns_indicators);
  return (
    <>
      <Paragraph title="项目简介" text={detail.ns_profile} />
      <Paragraph title="主要研究内容" text={detail.ns_content} />
      <section>
        <h3 className="font-medium text-ink-900">技术经济指标</h3>
        {indicators.length === 0 ? <p className="mt-1 text-ink-400">未填写量化指标</p> : <ul className="mt-1">{indicators.map((line) => <li key={line}>{line}</li>)}</ul>}
        <Paragraph title="社会效益说明" text={typeof detail.ns_indicators.benefit_note === "string" ? detail.ns_indicators.benefit_note : null} />
      </section>
      <RowList title="计划进度" rows={detail.ns_schedules.map((item) => [item.start_date, item.end_date, item.goal].filter(Boolean).join(" "))} />
      <RowList title="仪器设备" rows={detail.ns_equipments.map((item) => `${item.name_spec} × ${item.quantity}，单价 ${item.unit_price} 元`)} />
    </>
  );
}

const indicatorLabels: Array<[string, string]> = [
  ["paper_core", "核心期刊论文"],
  ["paper_general", "一般期刊论文"],
  ["paper_provincial", "省级期刊论文"],
  ["paper_domestic", "国内论文"],
  ["paper_international", "国际论文"],
  ["report_research", "研究报告"],
  ["monograph", "专著"],
  ["software_copyright_applied", "软件著作权申请"],
  ["patent_invention_applied", "发明专利申请"],
  ["patent_utility_applied", "实用新型申请"],
  ["new_product", "新产品"],
  ["new_technology", "新技术"],
  ["demo_site", "示范点"],
  ["test_base", "试验基地"],
];

function indicatorLines(indicators: ApplicationDetail["ns_indicators"]) {
  const lines = indicatorLabels.flatMap(([key, label]) => {
    const value = indicators[key];
    return typeof value === "number" && value > 0 ? [`${label} ${value}`] : [];
  });
  for (const [key, label] of [
    ["new_sales_income", "新增销售收入"],
    ["new_profit_tax", "新增利税"],
  ] as const) {
    const value = indicators[key];
    if (typeof value === "string" && filledAmount(value)) lines.push(`${label} ${value} 万元`);
  }
  return lines;
}

function Fact({ label, value }: { label: string; value: string | null }) {
  return (
    <div>
      <dt className="text-[11px] text-ink-400">{label}</dt>
      <dd>{value?.trim() || "—"}</dd>
    </div>
  );
}

function Paragraph({ title, text }: { title: string; text: string | null | undefined }) {
  if (!text?.trim()) return null;
  return (
    <section>
      <h3 className="font-medium text-ink-900">{title}</h3>
      <p className="mt-1 whitespace-pre-wrap">{text}</p>
    </section>
  );
}

function RowList({ title, rows }: { title: string; rows: string[] }) {
  const filled = rows.map((row) => row.trim()).filter(Boolean);
  if (filled.length === 0) return null;
  return (
    <section>
      <h3 className="font-medium text-ink-900">{title}</h3>
      <ul className="mt-1 list-disc pl-4">
        {filled.map((row) => (
          <li key={row}>{row}</li>
        ))}
      </ul>
    </section>
  );
}

function filledAmount(amount: string) {
  const value = Number(amount);
  return Number.isFinite(value) && value !== 0;
}

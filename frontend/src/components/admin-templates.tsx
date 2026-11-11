import { AdminPanel } from "@/components/admin-ui";
import { Badge } from "@/components/ui/badge";
import { StickyTable } from "@/components/ui/sticky-table";
import type { Meta, ProjectClass } from "@/lib/api";

const forms: Record<string, string> = {
  social: "社会科学类申报书",
  natural: "自然科学类申报书",
  dual_high: "自然科学类申报书",
};

export function AdminTemplates({ meta }: { meta: Meta | null }) {
  const classes: ProjectClass[] = meta?.project_classes ?? [];
  return (
    <div className="mt-4 space-y-4">
      <AdminPanel className="overflow-hidden">
        <div className="border-b border-ink-100 px-4 py-3">
          <h2 className="text-[14px] font-semibold text-ink-900">申报类别</h2>
          <p className="mt-1 text-[12px] text-ink-500">
            类别来自系统信息。经费科目 {meta ? meta.budget_subject_count : "—"} 项。申报书栏目由填报页固定，没有模板保存接口。
          </p>
        </div>
        <StickyTable
          rows={classes}
          rowKey={(item) => item.code}
          empty="系统信息未返回申报类别"
          columns={[
            { header: "类别", width: 200, cell: (item) => <span className="font-medium text-ink-900">{item.label}</span> },
            { header: "代码", width: 140, cell: (item) => <span className="text-ink-500">{item.code}</span> },
            { header: "申报书", width: 220, cell: (item) => forms[item.code] ?? item.form },
            { header: "当前批次", width: 220, cell: () => meta?.cycle ? `${meta.cycle.year} ${meta.cycle.title}` : "没有当前批次" },
            { header: "状态", width: 100, align: "right", cell: () => <Badge tone="teal">使用中</Badge> },
          ]}
        />
      </AdminPanel>
    </div>
  );
}

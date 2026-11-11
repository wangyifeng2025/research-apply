import { AdminPanel } from "@/components/admin-ui";
import { Badge } from "@/components/ui/badge";
import { StickyTable } from "@/components/ui/sticky-table";
import { roleOptions, type AdminUser, type RoleCode } from "@/lib/admin";

const descriptions: Record<RoleCode, string> = {
  teacher: "填写、保存、提交和撤回本人的申报书",
  department_reviewer: "审核本部门已提交的申报，结论为通过、退回或不通过",
  research_office: "审核全院已通过系审的申报，填写封面编号、指派专家并决定立项",
  expert: "对指派给自己的申报做匿名评审，只看研究内容并给出评分和结论",
  admin: "维护部门、用户和申报批次",
};

export function AdminRoles({ users }: { users: AdminUser[] }) {
  const rows = roleOptions.map((role) => ({
    ...role,
    count: users.filter((user) => user.roles.includes(role.code)).length,
    description: descriptions[role.code],
  }));

  return (
    <div className="mt-4">
      <AdminPanel className="overflow-hidden">
        <div className="border-b border-ink-100 px-4 py-3">
          <h2 className="text-[14px] font-semibold text-ink-900">系统角色</h2>
          <p className="mt-1 text-[12px] text-ink-500">角色由账号上的角色列表决定，在用户管理里分配。这里没有单独的角色保存接口。</p>
        </div>
        <StickyTable
          rows={rows}
          rowKey={(role) => role.code}
          columns={[
            { header: "角色", width: 160, cell: (role) => <span className="font-medium text-ink-900">{role.label}</span> },
            { header: "代码", width: 180, cell: (role) => <span className="text-ink-500">{role.code}</span> },
            { header: "账号数", width: 90, cell: (role) => role.count },
            { header: "说明", width: 420, cell: (role) => role.description },
            { header: "分配方式", width: 140, align: "right", cell: () => <Badge tone="neutral">在用户管理中设置</Badge> },
          ]}
        />
      </AdminPanel>
    </div>
  );
}

import { AdminPanel, AdminStat } from "@/components/admin-ui";
import { Badge } from "@/components/ui/badge";
import { StickyTable } from "@/components/ui/sticky-table";
import { roleLabel, type AdminUser, type Cycle, type Department } from "@/lib/admin";

export function AdminStats({ users, departments, cycles }: { users: AdminUser[]; departments: Department[]; cycles: Cycle[] }) {
  const expertCount = users.filter((user) => user.roles.includes("expert")).length;
  const openCycles = cycles.filter((cycle) => cycle.status === "open").length;
  const ranked = [...departments].sort((left, right) => right.member_count - left.member_count);
  const maxMembers = Math.max(1, ...ranked.map((item) => item.member_count));

  return (
    <div className="mt-4 space-y-4">
      <div className="grid grid-cols-2 gap-3 xl:grid-cols-4">
        <AdminStat label="系统用户" value={users.length} unit="人" icon="mdi:account-multiple-outline" foot={`启用 ${users.filter((user) => user.is_active).length} 人`} />
        <AdminStat label="部门" value={departments.length} unit="个" icon="mdi:office-building-cog-outline" />
        <AdminStat label="评审专家" value={expertCount} unit="人" icon="mdi:account-supervisor-outline" foot="按专家角色统计" />
        <AdminStat label="开放批次" value={openCycles} unit="个" icon="mdi:calendar-clock-outline" gold foot={`批次共 ${cycles.length} 个`} />
      </div>
      <div className="grid gap-4 xl:grid-cols-5">
        <AdminPanel className="p-4 xl:col-span-2">
          <h2 className="text-[14px] font-semibold text-ink-900">部门人数</h2>
          <p className="mt-0.5 text-[12px] text-ink-400">按已关联用户统计</p>
          <div className="mt-4 space-y-3">
            {ranked.length === 0 ? <p className="text-[12px] text-ink-400">还没有部门</p> : ranked.map((department) => (
              <div key={department.id} className="grid grid-cols-[120px_1fr_32px] items-center gap-3 text-[12px]">
                <span className="truncate text-ink-600">{department.name}</span>
                <div className="h-2 rounded-full bg-ink-50"><div className="h-2 rounded-full bg-teal-600" style={{ width: `${department.member_count / maxMembers * 100}%` }} /></div>
                <span className="text-right text-ink-500">{department.member_count}</span>
              </div>
            ))}
          </div>
        </AdminPanel>
        <AdminPanel className="overflow-hidden xl:col-span-3">
          <div className="border-b border-ink-100 px-4 py-3">
            <h2 className="text-[14px] font-semibold text-ink-900">账号角色</h2>
            <p className="mt-0.5 text-[12px] text-ink-400">一人可以有多个角色，因此人数可以大于用户总数</p>
          </div>
          <StickyTable
            rows={["teacher", "department_reviewer", "research_office", "expert", "admin"].map((code) => ({ code, count: users.filter((user) => user.roles.includes(code as AdminUser["roles"][number])).length }))}
            rowKey={(row) => row.code}
            columns={[
              { header: "角色", width: 160, cell: (row) => <span className="font-medium text-ink-900">{roleLabel(row.code)}</span> },
              { header: "代码", width: 180, cell: (row) => <span className="text-ink-500">{row.code}</span> },
              { header: "人数", width: 80, cell: (row) => row.count },
              { header: "占比", width: 160, cell: (row) => <span className="text-ink-500">{users.length === 0 ? "—" : `${Math.round(row.count / users.length * 100)}%`}</span> },
              { header: "来源", width: 120, align: "right", cell: () => <Badge tone="neutral">用户列表</Badge> },
            ]}
          />
        </AdminPanel>
      </div>
      <p className="text-[12px] text-ink-400">申报量、审核进度和操作日志还没有统计接口，这里不展示估算数字。</p>
    </div>
  );
}

import { apiSend } from "@/lib/api";

export const roleOptions = [
  { code: "teacher", label: "教师" },
  { code: "department_reviewer", label: "系部负责人" },
  { code: "research_office", label: "科研处管理员" },
  { code: "expert", label: "评审专家" },
  { code: "admin", label: "系统管理员" },
] as const;

export type RoleCode = (typeof roleOptions)[number]["code"];

export type Department = {
  id: string;
  name: string;
  code: string | null;
  member_count: number;
};

export type DepartmentInput = {
  name: string;
  code: string | null;
};

export type AdminUser = {
  id: string;
  username: string;
  real_name: string;
  employee_no: string | null;
  gender: string | null;
  ethnicity: string | null;
  admin_title: string | null;
  professional_title: string | null;
  research_expertise: string | null;
  education: string | null;
  degree: string | null;
  phone: string | null;
  email: string | null;
  department_id: string | null;
  department_name: string | null;
  is_active: boolean;
  roles: RoleCode[];
};

export type UserInput = {
  username: string;
  real_name: string;
  employee_no: string | null;
  gender: string | null;
  ethnicity: string | null;
  admin_title: string | null;
  professional_title: string | null;
  research_expertise: string | null;
  education: string | null;
  degree: string | null;
  phone: string | null;
  email: string | null;
  department_id: string | null;
  is_active: boolean;
  roles: RoleCode[];
};

export type CreateUserInput = UserInput & { password: string };

export type UserFilters = {
  department_id?: string;
  role?: string;
  keyword?: string;
};

export type CycleStatus = "draft" | "open" | "closed";

export type Cycle = {
  id: string;
  cycle_year: number;
  title: string;
  opens_at: string | null;
  closes_at: string | null;
  status: CycleStatus;
};

export type CycleInput = {
  cycle_year: number;
  title: string;
  opens_at: string | null;
  closes_at: string | null;
  status: CycleStatus;
};

export async function listDepartments(): Promise<Department[]> {
  return (await apiSend<{ items: Department[] }>("/api/departments")).items;
}

export function createDepartment(input: DepartmentInput): Promise<Department> {
  return apiSend("/api/admin/departments", { method: "POST", body: JSON.stringify(input) });
}

export function updateDepartment(id: string, input: DepartmentInput): Promise<Department> {
  return apiSend(`/api/admin/departments/${id}`, { method: "PATCH", body: JSON.stringify(input) });
}

export function deleteDepartment(id: string): Promise<void> {
  return apiSend(`/api/admin/departments/${id}`, { method: "DELETE" });
}

export async function listUsers(filters: UserFilters = {}): Promise<AdminUser[]> {
  const params = new URLSearchParams();
  Object.entries(filters).forEach(([key, value]) => {
    if (value) params.set(key, value);
  });
  const suffix = params.size > 0 ? `?${params.toString()}` : "";
  return (await apiSend<{ items: AdminUser[] }>(`/api/admin/users${suffix}`)).items;
}

export function createUser(input: CreateUserInput): Promise<AdminUser> {
  return apiSend("/api/admin/users", { method: "POST", body: JSON.stringify(input) });
}

export function updateUser(id: string, input: Partial<UserInput>): Promise<AdminUser> {
  return apiSend(`/api/admin/users/${id}`, { method: "PATCH", body: JSON.stringify(input) });
}

export function deleteUser(id: string): Promise<void> {
  return apiSend(`/api/admin/users/${id}`, { method: "DELETE" });
}

export function resetUserPassword(id: string, password: string): Promise<void> {
  return apiSend(`/api/admin/users/${id}/reset-password`, {
    method: "POST",
    body: JSON.stringify({ password }),
  });
}

export async function listCycles(): Promise<Cycle[]> {
  return (await apiSend<{ items: Cycle[] }>("/api/cycles")).items;
}

export function createCycle(input: CycleInput): Promise<Cycle> {
  return apiSend("/api/cycles", { method: "POST", body: JSON.stringify(input) });
}

export function updateCycle(id: string, input: Partial<CycleInput>): Promise<Cycle> {
  return apiSend(`/api/cycles/${id}`, { method: "PATCH", body: JSON.stringify(input) });
}

export function roleLabel(role: string): string {
  return roleOptions.find((item) => item.code === role)?.label ?? role;
}

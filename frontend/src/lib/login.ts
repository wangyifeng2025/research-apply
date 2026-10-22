import { z } from "zod";

export const loginRoles = ["teacher", "department", "office", "expert", "admin"] as const;

export type LoginRole = (typeof loginRoles)[number];

export const loginSchema = z.object({
  role: z.enum(loginRoles, { error: "请选择登录角色" }),
  username: z.string().trim().min(1, "请输入工号或账号"),
  password: z.string().min(1, "请输入登录密码"),
  remember: z.boolean(),
});

export type LoginValues = z.infer<typeof loginSchema>;

export const loginStorageKey = "research-apply.login.v1";

export type StoredLogin = {
  v: 1;
  username: string;
  role: LoginRole;
  realName?: string;
  departmentName?: string | null;
};

export const roleNav: Record<LoginRole, { id: string; label: string }[]> = {
  teacher: [
    { id: "desk", label: "工作台" },
    { id: "apply", label: "项目申报" },
  ],
  department: [{ id: "review", label: "系部初审" }],
  office: [
    { id: "review", label: "科研处复核" },
    { id: "publicity", label: "立项公示" },
  ],
  expert: [{ id: "review", label: "专家盲审" }],
  admin: [{ id: "console", label: "系统管理" }],
};

export const backendRoles: Record<LoginRole, string> = {
  teacher: "teacher",
  department: "department_reviewer",
  office: "research_office",
  expert: "expert",
  admin: "admin",
};

export const roleHomes: Record<LoginRole, string> = {
  teacher: "/applications",
  department: "/reviews/department",
  office: "/reviews/office",
  expert: "/reviews/expert",
  admin: "/admin",
};

type RolePreset = {
  label: string;
  mark: string;
  summary: string;
  username: string;
  password: string;
  hint: string;
  note: string;
};

export const rolePresets: Record<LoginRole, RolePreset> = {
  teacher: {
    label: "教师",
    mark: "教",
    summary: "填写申报 · 查看进度",
    username: "zhaomingyuan",
    password: "research2026",
    hint: "赵明远 · 经济管理学院",
    note: "进入我的申报，填写社会科学或自然科学申报书。",
  },
  department: {
    label: "系审核人",
    mark: "系",
    summary: "本部门审核",
    username: "liwenbo",
    password: "dept2026",
    hint: "李文博 · 经济管理学院",
    note: "进入系审核，处理本部门已提交的申报。",
  },
  office: {
    label: "科研处",
    mark: "科",
    summary: "复核 · 分配盲审",
    username: "wanghaining",
    password: "office2026",
    hint: "王海宁 · 科研处",
    note: "进入科研处审核，填写封面编号并分配专家。",
  },
  expert: {
    label: "评审专家",
    mark: "专",
    summary: "匿名盲审",
    username: "expert-e016",
    password: "review2026",
    hint: "匿名专家 E-016",
    note: "进入专家盲审。申请人看不到专家身份。",
  },
  admin: {
    label: "系统管理员",
    mark: "管",
    summary: "用户 · 部门 · 批次",
    username: "zhoujin",
    password: "admin2026",
    hint: "周谨 · 系统管理员",
    note: "登录后进入系统管理。",
  },
};

export function readStoredLogin(): StoredLogin | null {
  if (typeof window === "undefined") {
    return null;
  }
  return readStored(localStorage);
}

export function readActiveLogin(): StoredLogin | null {
  if (typeof window === "undefined") {
    return null;
  }
  const session = readStored(sessionStorage);
  return session ?? readStoredLogin();
}

function readStored(storage: Storage): StoredLogin | null {
  try {
    const raw = storage.getItem(loginStorageKey);
    if (!raw) {
      return null;
    }
    return parseStoredLogin(raw);
  } catch {
    return null;
  }
}

function parseStoredLogin(raw: string): StoredLogin | null {
  try {
    const parsed: unknown = JSON.parse(raw);
    if (
      typeof parsed === "object" &&
      parsed !== null &&
      "v" in parsed &&
      parsed.v === 1 &&
      "username" in parsed &&
      typeof parsed.username === "string" &&
      "role" in parsed &&
      typeof parsed.role === "string" &&
      loginRoles.includes(parsed.role as LoginRole)
    ) {
      const realName =
        "realName" in parsed && typeof parsed.realName === "string" ? parsed.realName : undefined;
      const departmentName =
        "departmentName" in parsed &&
        (typeof parsed.departmentName === "string" || parsed.departmentName === null)
          ? parsed.departmentName
          : undefined;
      return {
        v: 1,
        username: parsed.username,
        role: parsed.role as LoginRole,
        realName,
        departmentName,
      };
    }
  } catch {
    return null;
  }
  return null;
}

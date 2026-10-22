export type WorkflowStep = {
  code: string;
  label: string;
};

export type ProjectClass = {
  code: string;
  label: string;
  form: string;
};

export type CycleInfo = {
  year: number;
  title: string;
  status: string;
};

export type Meta = {
  service: string;
  database: string;
  workflow: WorkflowStep[];
  project_classes: ProjectClass[];
  cycle: CycleInfo | null;
  budget_subject_count: number;
};

export type Health = {
  status: string;
  database: string;
};

const apiBase = process.env.NEXT_PUBLIC_API_BASE ?? "http://127.0.0.1:8080";

export function apiUrl(path: string): string {
  return `${apiBase}${path}`;
}

export async function fetchHealth(): Promise<Health | null> {
  try {
    const response = await fetch(apiUrl("/health"), { cache: "no-store" });
    if (!response.ok) {
      return null;
    }
    return (await response.json()) as Health;
  } catch {
    return null;
  }
}

export async function fetchMeta(): Promise<Meta | null> {
  try {
    const response = await fetch(apiUrl("/api/meta"), { cache: "no-store" });
    if (!response.ok) {
      return null;
    }
    return (await response.json()) as Meta;
  } catch {
    return null;
  }
}

export type AuthUser = {
  id: string;
  username: string;
  real_name: string;
  roles: string[];
  department_name: string | null;
};

export type FieldIssue = { field: string; message: string };

export class ApiError extends Error {
  status: number;
  fields: FieldIssue[];

  constructor(status: number, message: string, fields: FieldIssue[] = []) {
    super(message);
    this.status = status;
    this.fields = fields;
  }
}

export async function loginRequest(username: string, password: string): Promise<AuthUser> {
  let response: Response;
  try {
    response = await fetch("/api/auth/login", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      credentials: "include",
      body: JSON.stringify({ username, password }),
    });
  } catch {
    throw new ApiError(0, "无法连接服务器，请确认后端已启动");
  }

  const body = (await response.json().catch(() => null)) as { error?: string; user?: AuthUser } | null;
  if (!response.ok || !body?.user) {
    throw new ApiError(response.status, body?.error ?? "登录失败");
  }
  return body.user;
}

type ErrorBody = { error?: string; fields?: FieldIssue[] };

export async function apiSend<T>(path: string, init: RequestInit = {}): Promise<T> {
  const headers = new Headers(init.headers);
  if (init.body && !headers.has("Content-Type")) {
    headers.set("Content-Type", "application/json");
  }
  let response: Response;
  try {
    response = await fetch(path, { ...init, headers, credentials: "include" });
  } catch {
    throw new ApiError(0, "无法连接服务器，请确认后端已启动");
  }
  if (response.status === 204) {
    return undefined as T;
  }
  const body = (await response.json().catch(() => null)) as (ErrorBody & T) | null;
  if (!response.ok) {
    const fields = body?.fields?.filter((item) => item.message) ?? [];
    const message = fields.length > 0 ? fields.map((item) => item.message).join("；") : (body?.error ?? "请求失败");
    throw new ApiError(response.status, message, fields);
  }
  return body as T;
}

export function cycleStatusLabel(status: string): string {
  switch (status) {
    case "open":
      return "申报中";
    case "closed":
      return "已截止";
    default:
      return "未开放";
  }
}

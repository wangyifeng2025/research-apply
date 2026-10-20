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

import type { TimelineEvent } from "@/lib/timeline";

const stageLabels: Record<string, string> = {
  department: "系审核",
  research_office: "科研处审核",
  expert: "专家盲审",
  conclusion: "立项结论",
};

export function ReviewTimeline({ events }: { events: TimelineEvent[] }) {
  if (events.length === 0) {
    return <p className="text-[12.5px] text-ink-400">还没有审核记录。</p>;
  }
  return (
    <ol>
      {events.map((event, index) => (
        <li key={`${event.reviewed_at}-${event.stage}-${index}`} className="flex gap-3">
          <div className="flex flex-col items-center">
            <span className="mt-1.5 h-2.5 w-2.5 shrink-0 rounded-full bg-teal-600" />
            {index < events.length - 1 ? <span className="my-1 w-px flex-1 bg-teal-200" /> : null}
          </div>
          <div className="min-w-0 pb-4">
            <p className="text-[12.5px] font-medium text-ink-800">
              {stageLabels[event.stage] ?? event.stage} · {decisionLabel(event)}
            </p>
            <p className="mt-0.5 text-[11.5px] text-ink-500">
              {event.actor} · {formatTime(event.reviewed_at)}
              {event.score ? ` · ${event.score} 分` : ""}
            </p>
            {event.opinion ? <p className="mt-1 text-[12.5px] leading-5 whitespace-pre-wrap text-ink-700">{event.opinion}</p> : null}
            <DepartmentChecks event={event} />
          </div>
        </li>
      ))}
    </ol>
  );
}

function decisionLabel(event: TimelineEvent) {
  if (event.stage === "expert") {
    if (event.decision === "approved") return "建议立项";
    if (event.decision === "rejected") return "不建议立项";
  }
  const labels: Record<string, string> = {
    approved: "通过",
    rejected: "不通过",
    returned: "退回",
    established: "立项",
    expert_rejected: "不予立项",
  };
  return labels[event.decision] ?? event.decision;
}

function DepartmentChecks({ event }: { event: TimelineEvent }) {
  if (event.stage !== "department") return null;
  const items = [
    ["情况属实", event.facts_accurate],
    ["团队具备", event.team_qualified],
    ["条件具备", event.resources_ready],
    ["同意管理", event.agree_to_manage],
  ] as const;
  if (items.every(([, value]) => value === null)) return null;
  return (
    <p className="mt-1 text-[11.5px] text-ink-500">
      {items.map(([label, value]) => `${label}${value ? "是" : "否"}`).join("，")}
    </p>
  );
}

function formatTime(value: string) {
  return value.slice(0, 16).replace("T", " ");
}

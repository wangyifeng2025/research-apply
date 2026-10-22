"use client";

import { useEffect, useRef, useState, type ReactNode } from "react";

import { Icon } from "@/components/icon";
import {
  classLabels,
  createApplication,
  deleteDraft,
  fetchApplication,
  saveSection,
  statusLabels,
  submitApplication,
  withdrawApplication,
  type ApplicationDetail,
  type EquipmentRow,
  type MemberInput,
  type OutputRow,
  type ProjectClassCode,
  type ScheduleRow,
  type TextRow,
} from "@/lib/applications";
import { ApiError, type FieldIssue } from "@/lib/api";
import { checkBasicSection } from "@/lib/checks";

type Props = {
  applicationId: string | null;
  onOpen: (id: string) => void;
  onClose: () => void;
  onChanged: () => void;
};

const socialSteps = [
  { id: "basic", mark: "一", title: "基本情况" },
  { id: "works", mark: "二", title: "近五年成果" },
  { id: "design", mark: "三", title: "项目设计论证" },
  { id: "base", mark: "四", title: "研究基础" },
  { id: "output", mark: "五", title: "预期成果" },
  { id: "budget", mark: "六", title: "经费预算" },
  { id: "recommend", mark: "七", title: "推荐人意见" },
  { id: "dept", mark: "八", title: "部门意见" },
  { id: "preview", mark: "览", title: "预览提交" },
];

const naturalSteps = [
  { id: "basic", mark: "一", title: "基本情况" },
  { id: "content", mark: "二", title: "主要研究内容" },
  { id: "indicator", mark: "三", title: "技术经济指标" },
  { id: "plan", mark: "四", title: "计划进度" },
  { id: "equip", mark: "五", title: "仪器设备" },
  { id: "budget", mark: "六", title: "经费预算" },
  { id: "recommend", mark: "七", title: "推荐人意见" },
  { id: "dept", mark: "八", title: "部门意见" },
  { id: "preview", mark: "览", title: "预览提交" },
];

const commitmentText = "我承诺对本人填写的各项内容的真实性负责，保证没有知识产权争议。如获准立项，我承诺以本表为有约束力的协议，遵守黔西南民族职业技术学院科研项目管理的相关规定，按计划认真开展研究工作，取得预期研究成果。学院有权使用本表所有数据和资料，并有权留存、报送、公布和推广项目研究成果。";

function blankMember(): MemberInput {
  return { name: "", birth_date: "", professional_title: "", research_expertise: "", education: "", degree: "", organization: "" };
}

export function TeacherApply({ applicationId, onOpen, onClose, onChanged }: Props) {
  const [detail, setDetail] = useState<ApplicationDetail | null>(null);
  const [step, setStep] = useState(0);
  const [busy, setBusy] = useState(false);
  const [message, setMessage] = useState("");
  const [issues, setIssues] = useState<FieldIssue[]>([]);
  const issueRef = useRef<HTMLDivElement>(null);
  const [title, setTitle] = useState("");
  const [keywords, setKeywords] = useState("");
  const [filledOn, setFilledOn] = useState("");
  const [gender, setGender] = useState("");
  const [ethnicity, setEthnicity] = useState("");
  const [birthDate, setBirthDate] = useState("");
  const [adminTitle, setAdminTitle] = useState("");
  const [professionalTitle, setProfessionalTitle] = useState("");
  const [researchExpertise, setResearchExpertise] = useState("");
  const [education, setEducation] = useState("");
  const [degree, setDegree] = useState("");
  const [mobilePhone, setMobilePhone] = useState("");
  const [members, setMembers] = useState<MemberInput[]>([blankMember()]);
  const [achievements, setAchievements] = useState<TextRow[]>([]);
  const [design, setDesign] = useState({ topic_basis: "", research_content: "", methodology: "", innovation: "", expected_outcome: "", reference_list: "" });
  const [foundation, setFoundation] = useState({ academic_resume: "", research_basis: "", undertaken_projects: "", support_conditions: "" });
  const [stage, setStage] = useState<OutputRow[]>([]);
  const [finals, setFinals] = useState<OutputRow[]>([]);
  const [summary, setSummary] = useState("");
  const [researchContent, setResearchContent] = useState("");
  const [indicators, setIndicators] = useState<Record<string, number | string | null>>({});
  const [schedules, setSchedules] = useState<ScheduleRow[]>([]);
  const [equipments, setEquipments] = useState<EquipmentRow[]>([]);
  const [budget, setBudget] = useState<ApplicationDetail["budget"]["lines"]>([]);
  const [recommender, setRecommender] = useState({ name: "", professional_title: "", expertise: "", organization: "", opinion: "", signed_on: "" });
  const [commitment, setCommitment] = useState(false);

  useEffect(() => {
    if (!applicationId) {
      setDetail(null);
      return;
    }
    let cancelled = false;
    void fetchApplication(applicationId)
      .then((view) => {
        if (cancelled) return;
        applyView(view);
      })
      .catch((error: unknown) => {
        if (!cancelled) setMessage(error instanceof ApiError ? error.message : "申报书加载失败");
      });
    return () => {
      cancelled = true;
    };
    function applyView(view: ApplicationDetail) {
      setDetail(view);
      setStep(0);
      setTitle(view.title ?? "");
      setKeywords(view.keywords ?? "");
      setFilledOn(view.filled_on ?? "");
      setGender(view.gender ?? "");
      setEthnicity(view.ethnicity ?? "");
      setBirthDate(view.birth_date ?? "");
      setAdminTitle(view.admin_title ?? "");
      setProfessionalTitle(view.professional_title ?? "");
      setResearchExpertise(view.research_expertise ?? "");
      setEducation(view.education ?? "");
      setDegree(view.degree ?? "");
      setMobilePhone(view.mobile_phone ?? "");
      setMembers(view.members.length > 0 ? view.members.map((row) => ({
        name: row.name,
        birth_date: row.birth_date ?? "",
        professional_title: row.professional_title ?? "",
        research_expertise: row.research_expertise ?? "",
        education: row.education ?? "",
        degree: row.degree ?? "",
        organization: row.organization ?? "",
      })) : [blankMember()]);
      setAchievements(view.ss_achievements.map((row) => ({
        title: row.title,
        researchers: row.researchers ?? "",
        outcome_form: row.outcome_form ?? "",
        publication: row.publication ?? "",
        published_on: row.published_on ?? "",
      })));
      setDesign({
        topic_basis: view.ss_design.topic_basis ?? "",
        research_content: view.ss_design.research_content ?? "",
        methodology: view.ss_design.methodology ?? "",
        innovation: view.ss_design.innovation ?? "",
        expected_outcome: view.ss_design.expected_outcome ?? "",
        reference_list: view.ss_design.reference_list ?? "",
      });
      setFoundation({
        academic_resume: view.ss_foundation.academic_resume ?? "",
        research_basis: view.ss_foundation.research_basis ?? "",
        undertaken_projects: view.ss_foundation.undertaken_projects ?? "",
        support_conditions: view.ss_foundation.support_conditions ?? "",
      });
      setStage(view.ss_stage_outputs.map(fillOutput));
      setFinals(view.ss_final_outputs.map(fillOutput));
      setSummary(view.ns_profile ?? "");
      setResearchContent(plainContent(view.ns_content ?? ""));
      const sheet = view.ns_indicators as Record<string, unknown>;
      const counts: Record<string, number | string | null> = {};
      for (const [key, value] of Object.entries(sheet)) {
        if (typeof value === "number" || typeof value === "string" || value === null) counts[key] = value;
      }
      setIndicators(counts);
      setSchedules(view.ns_schedules.map((row) => ({ start_date: row.start_date ?? "", end_date: row.end_date ?? "", goal: row.goal ?? "" })));
      setEquipments(view.ns_equipments.map((row) => ({
        name_spec: row.name_spec,
        quantity: String(row.quantity),
        unit_price: row.unit_price,
        funding_source: row.funding_source ?? "",
        purpose: row.purpose ?? "",
      })));
      setBudget(view.budget.lines);
      setRecommender({
        name: view.recommender?.name ?? "",
        professional_title: view.recommender?.professional_title ?? "",
        expertise: view.recommender?.expertise ?? "",
        organization: view.recommender?.organization ?? "",
        opinion: view.recommender?.opinion ?? "",
        signed_on: view.recommender?.signed_on ?? "",
      });
      setCommitment(view.commitment_accepted);
      setMessage("");
    }
  }, [applicationId]);

  useEffect(() => {
    if (issues.length > 0) issueRef.current?.scrollIntoView({ block: "nearest", behavior: "smooth" });
  }, [issues]);

  function issueOf(field: string) {
    return issues.find((item) => item.field === field)?.message;
  }

  function clearIssue(field: string) {
    setIssues((current) => current.filter((item) => item.field !== field));
  }

  const steps = detail?.project_class === "social" ? socialSteps : naturalSteps;
  const current = steps[step];
  const locked = detail ? !detail.editable : false;

  async function choose(projectClass: ProjectClassCode) {
    setBusy(true);
    setMessage("");
    try {
      const view = await createApplication(projectClass);
      onChanged();
      onOpen(view.id);
    } catch (error) {
      setMessage(error instanceof ApiError ? error.message : "新建申报失败");
    } finally {
      setBusy(false);
    }
  }

  async function leave() {
    if (detail?.editable && current && current.id !== "preview" && current.id !== "dept") {
      const saved = await persist(false);
      if (!saved) return;
    }
    onClose();
  }

  async function jump(index: number) {
    if (index === step) return;
    if (detail?.editable && current && current.id !== "preview" && current.id !== "dept") {
      const saved = await persist(false);
      if (!saved) return;
    }
    setStep(index);
  }

  async function persist(advance: boolean) {
    if (!detail || !current || locked) {
      if (advance) setStep((value) => Math.min(steps.length - 1, value + 1));
      return true;
    }
    setBusy(true);
    setMessage("");
    try {
      if (current.id !== "preview" && current.id !== "dept") {
        if (current.id === "basic") {
          const found = checkBasicSection({
            title, keywords, filledOn, birthDate, gender, ethnicity, adminTitle, professionalTitle,
            researchExpertise, education, degree, mobilePhone, members,
          });
          if (found.length > 0) {
            setIssues(found);
            return false;
          }
        }
        setIssues([]);
        await saveCurrent(detail.id, current.id);
        onChanged();
        setMessage(advance ? "当前栏目已保存" : "草稿已保存");
      }
      if (advance && step < steps.length - 1) {
        setStep(step + 1);
      }
      return true;
    } catch (error) {
      if (error instanceof ApiError && error.fields.length > 0) {
        setIssues(error.fields);
        setMessage("");
      } else {
        setIssues([]);
        setMessage(error instanceof ApiError ? error.message : "保存失败");
      }
      return false;
    } finally {
      setBusy(false);
    }
  }

  async function saveCurrent(id: string, section: string) {
    if (section === "basic") {
      await saveSection(id, "basic", {
        title, keywords, filled_on: filledOn, gender, ethnicity, birth_date: birthDate,
        admin_title: adminTitle, professional_title: professionalTitle, research_expertise: researchExpertise,
        education, degree, mobile_phone: mobilePhone,
      });
      await saveSection(id, "members", { members });
      if (detail?.project_class !== "social") {
        await saveSection(id, "ns-profile", { summary });
      }
      return;
    }
    if (section === "works") {
      await saveSection(id, "ss-achievements", { rows: achievements });
      return;
    }
    if (section === "design") {
      await saveSection(id, "ss-design", design);
      return;
    }
    if (section === "base") {
      await saveSection(id, "ss-foundation", foundation);
      return;
    }
    if (section === "output") {
      await saveSection(id, "ss-outputs", { stage, final: finals });
      return;
    }
    if (section === "profile") {
      await saveSection(id, "ns-profile", { summary });
      return;
    }
    if (section === "content") {
      await saveSection(id, "ns-content", { content: researchContent });
      return;
    }
    if (section === "indicator") {
      await saveSection(id, "ns-indicators", indicators);
      return;
    }
    if (section === "plan") {
      await saveSection(id, "ns-schedules", { rows: schedules });
      return;
    }
    if (section === "equip") {
      await saveSection(id, "ns-equipments", {
        rows: equipments.filter((row) => row.name_spec.trim()).map((row) => ({
          ...row,
          quantity: Number(row.quantity || 0),
        })),
      });
      return;
    }
    if (section === "budget") {
      await saveSection(id, "budget", {
        lines: budget.filter((line) => line.fillable).map((line) => ({ subject_code: line.code, amount: line.amount || "0" })),
      });
      return;
    }
    if (section === "recommend") {
      await saveSection(id, "recommender", recommender);
    }
  }

  async function submit() {
    if (!detail) return;
    if (!commitment) {
      setMessage("提交前请勾选申请者承诺");
      return;
    }
    setBusy(true);
    try {
      const view = await submitApplication(detail.id);
      setDetail(view);
      setMessage("申报书已提交，等待系部审核");
      onChanged();
    } catch (error) {
      if (error instanceof ApiError && error.fields.length > 0) {
        setIssues(error.fields);
        setMessage("");
      } else {
        setIssues([]);
        setMessage(error instanceof ApiError ? error.message : "提交失败");
      }
    } finally {
      setBusy(false);
    }
  }

  async function withdraw() {
    if (!detail) return;
    setBusy(true);
    try {
      const view = await withdrawApplication(detail.id);
      setDetail(view);
      setMessage("已撤回为草稿，可以继续修改");
      onChanged();
    } catch (error) {
      setMessage(error instanceof ApiError ? error.message : "撤回失败");
    } finally {
      setBusy(false);
    }
  }

  async function removeDraft() {
    if (!detail || detail.status !== "draft") return;
    if (!window.confirm("删除草稿后，参加者、经费和各栏目都会从数据库一并删除。")) return;
    setBusy(true);
    try {
      await deleteDraft(detail.id);
      onChanged();
      onClose();
    } catch (error) {
      setMessage(error instanceof ApiError ? error.message : "删除失败");
      setBusy(false);
    }
  }

  if (applicationId && !detail) {
    return <p className="text-[13px] text-ink-500">{message || "正在打开申报书…"}</p>;
  }

  if (!applicationId || !detail) {
    return (
      <div>
        <div className="flex items-center gap-1.5 text-[11px] text-ink-400">
          <span>项目申报</span>
          <Icon name="mdi:chevron-right" className="text-xs" />
          <span className="text-ink-500">在线填报</span>
        </div>
        <h1 className="mt-1.5 text-[22px] font-semibold tracking-tight text-ink-900">新建申报</h1>
        <p className="mt-1 text-[12.5px] text-ink-500">点选类别后立即生成草稿，并进入对应申报书。社会科学类按附件 2，自然科学类和双高专项按附件 3。</p>
        <div className="mt-4 grid grid-cols-1 gap-4 xl:grid-cols-3">
          <section className="rounded-xl border border-ink-200/80 bg-white p-4 shadow-sm sm:p-5 xl:col-span-2">
            <div className="flex items-start justify-between gap-3">
              <div>
                <h2 className="text-[14px] font-semibold text-ink-900">选择申报类别</h2>
                <p className="mt-1 text-[11.5px] text-ink-500">社会科学类进入社科申报书，自然科学类进入自科申报书。双高专项使用自科申报书，类别单独记录。</p>
              </div>
              <span className="rounded bg-ink-50 px-2 py-1 text-[10px] text-ink-500">必选</span>
            </div>
            <div className="mt-4 grid gap-3 sm:grid-cols-2">
              <CategoryCard icon="mdi:book-open-page-variant-outline" tone="gold" title="社会科学类（B. 社会科学类）" text="经济管理、教育学、法学、社会学、文学与新闻传播等人文社科方向。" chips={["申报书七章", "近五年成果", "设计论证"]} action="进入社会科学类申报书" onClick={() => void choose("social")} disabled={busy} />
              <CategoryCard icon="mdi:atom" tone="teal" title="自然科学类（A. 自然科学类）" text="工程技术、信息科学、材料化工、生物医药与数学物理等理工方向。" chips={["申报书八章", "技术经济指标", "仪器设备"]} action="进入自然科学类申报书" onClick={() => void choose("natural")} disabled={busy} />
              <CategoryCard icon="mdi:flask-outline" tone="teal" title="双高专项（D. 双高校建设专项）" text="栏目与自然科学类申报书相同，项目类别记为双高专项。" chips={["使用自科申报书", "类别单独记录"]} action="进入双高专项申报书" onClick={() => void choose("dual_high")} disabled={busy} />
            </div>
            <div className="mt-3 rounded-lg border border-teal-100 bg-teal-50/50 p-3.5">
              <p className="text-[12px] font-medium text-ink-800">黔西南民族职业技术学院科技计划项目 · 2026 年</p>
              <p className="mt-2 text-[10.5px] leading-5 text-ink-500">编号由科研处统一编排。封面栏目在「一、基本情况」填写。主要参加者最多 5 人，不含负责人，也不含科研管理、财务和后勤人员。</p>
            </div>
            {message ? <p className="mt-3 text-[12px] text-red-600">{message}</p> : null}
          </section>
          <section className="rounded-xl border border-ink-200/80 bg-white p-4 shadow-sm">
            <div className="flex items-center justify-between">
              <h2 className="text-[13.5px] font-semibold text-ink-900">申报流程</h2>
              <span className="text-[10px] text-ink-400">共 6 个环节</span>
            </div>
            <ol className="mt-3">
              {["教师填报", "系部初审", "专家匿名盲审", "科研处复核", "学术委员会终审", "立项公示与归档"].map((label, index, list) => (
                <li key={label} className="flex gap-2.5">
                  <div className="flex flex-col items-center">
                    <span className={`flex h-5 w-5 items-center justify-center rounded-full text-[9px] font-semibold ${index === 0 ? "bg-teal-600 text-white" : "border border-ink-200 bg-white text-ink-500"}`}>{index + 1}</span>
                    {index < list.length - 1 ? <span className="my-1 w-px flex-1 bg-ink-200" /> : null}
                  </div>
                  <p className="pb-3 text-[11.5px] font-medium text-ink-800">{label}</p>
                </li>
              ))}
            </ol>
          </section>
        </div>
      </div>
    );
  }

  const direct = sumBudget(budget, "direct");
  const indirect = budget.filter((line) => line.group_name === "indirect" && !line.parent_code).reduce((sum, line) => sum + Number(line.amount || 0), 0);

  return (
    <div>
      <div className="flex flex-wrap items-end justify-between gap-3">
        <div>
          <p className="text-[11px] text-ink-400">{classLabels[detail.project_class]} · {statusLabels[detail.status] ?? detail.status}</p>
          <h1 className="mt-1 text-[22px] font-semibold tracking-tight text-ink-900">{title || "未命名申报书"}</h1>
        </div>
        <div className="flex gap-2">
          {detail.status === "draft" ? (
            <button type="button" onClick={() => void removeDraft()} className="inline-flex h-8 items-center gap-1 rounded-lg border border-red-200 px-3 text-[12px] text-red-600">删除草稿</button>
          ) : null}
          {detail.status === "submitted" ? (
            <button type="button" onClick={() => void withdraw()} className="inline-flex h-8 items-center gap-1 rounded-lg border border-ink-200 px-3 text-[12px] text-ink-600">撤回</button>
          ) : null}
          <button type="button" onClick={() => void leave()} className="inline-flex h-8 items-center gap-1 rounded-lg border border-ink-200 px-3 text-[12px] text-ink-600">返回列表</button>
        </div>
      </div>
      <div className="mt-4 rounded-xl border border-ink-200/80 bg-white px-3 py-3 shadow-sm sm:px-5 sm:py-4">
        <div className="mb-3 flex flex-wrap items-center gap-2 text-[11px] text-ink-500">
          <Icon name="mdi:file-document-outline" className="text-sm text-teal-600" />
          申报书模板
          <span className="rounded bg-teal-50 px-2 py-0.5 text-[10.5px] font-medium text-teal-700">{detail.project_class === "social" ? "B. 社会科学类" : detail.project_class === "dual_high" ? "D. 双高校建设专项" : "A. 自然科学类"}</span>
          <span className="ml-auto rounded-full bg-ink-50 px-2.5 py-1 text-[10.5px]">当前第 {step + 1} / {steps.length} 步</span>
        </div>
      <ol className="flex gap-2 overflow-x-auto pb-1">
        {steps.map((item, index) => (
          <li key={item.id}>
            <button type="button" onClick={() => void jump(index)} className={`flex min-w-[118px] items-center gap-2 rounded-lg border px-2.5 py-2 text-left ${index === step ? "border-teal-300 bg-teal-50" : "border-ink-200 bg-white"}`}>
              <span className={`flex h-6 w-6 items-center justify-center rounded text-[10px] font-semibold ${index <= step ? "bg-teal-700 text-white" : "bg-ink-100 text-ink-500"}`}>{item.mark}</span>
              <span className="truncate text-[11.5px] text-ink-700">{item.title}</span>
            </button>
          </li>
        ))}
      </ol>
      </div>
      {issues.length > 0 ? (
        <div ref={issueRef} className="mt-3 rounded-lg border border-cinnabar/30 bg-red-50 px-3 py-2 text-[12px] text-cinnabar">
          <p className="font-medium">有 {issues.length} 处需要修改</p>
          <ul className="mt-1 list-disc space-y-0.5 pl-4">
            {issues.map((item) => <li key={`${item.field}-${item.message}`}>{item.message}</li>)}
          </ul>
        </div>
      ) : null}
      {message ? <p className="mt-3 rounded-lg bg-teal-50 px-3 py-2 text-[12px] text-teal-800">{message}</p> : null}

      <div className="mt-4">
        {current?.id === "basic" ? (
          <Chapter mark="一" title="基本情况" aside="对应申报书「一、基本情况」">
            <div className="grid gap-3.5 sm:grid-cols-2">
              <label className="block sm:col-span-2">
                <span className="field-label">项目名称</span>
                <input className="field-input" maxLength={40} placeholder="应准确、简明反映研究内容，最多不超过 40 个汉字" value={title} disabled={locked} aria-invalid={issueOf("title") ? true : undefined} onChange={(event) => { clearIssue("title"); setTitle(event.target.value); }} />
                <p className="sheet-hint mt-1">最多不超过 40 个汉字，当前 {title.length}/40 字</p>
                {issueOf("title") ? <span className="mt-1 block text-[11px] text-cinnabar">{issueOf("title")}</span> : null}
              </label>
              <label className="block sm:col-span-2">
                <span className="field-label">关键词</span>
                <input className="field-input" placeholder="最多不超过 5 个，各词之间用分号隔开" value={keywords} disabled={locked} aria-invalid={issueOf("keywords") ? true : undefined} onChange={(event) => { clearIssue("keywords"); setKeywords(event.target.value); }} />
                <p className="sheet-hint mt-1">最多不超过 5 个，各词之间用分号隔开</p>
                {issueOf("keywords") ? <span className="mt-1 block text-[11px] text-cinnabar">{issueOf("keywords")}</span> : null}
              </label>
              <ReadOnly label="项目类别" value={detail.project_class === "social" ? "B. 社会科学类" : detail.project_class === "dual_high" ? "D. 双高校建设专项" : "A. 自然科学类"} />
              <ReadOnly label="负责人所在部门" value={detail.department_name ?? "未分配部门"} />
              <Field name="filled_on" label="填表日期" type="date" value={filledOn} disabled={locked} issue={issueOf("filled_on")} onClear={clearIssue} onChange={setFilledOn} />
              <ReadOnly label="联系电话" value={mobilePhone || "在下方项目负责人中填写"} />
            </div>
            <div className="mt-5">
              <div className="flex items-center gap-2">
                <p className="text-[12.5px] font-semibold text-ink-800">项目负责人</p>
                <span className="rounded bg-ink-100 px-1.5 py-0.5 text-[10px] text-ink-500">姓名和部门取自账号，其余只保存在这份申报书</span>
              </div>
              <div className="mt-2.5 grid gap-3 border-t border-ink-100 pt-3.5 sm:grid-cols-2 lg:grid-cols-3">
                <ReadOnly label="姓名" value={detail.principal_name ?? ""} />
                <Field name="gender" label="性别" value={gender} disabled={locked} issue={issueOf("gender")} onClear={clearIssue} onChange={setGender} />
                <Field name="ethnicity" label="民族" value={ethnicity} disabled={locked} issue={issueOf("ethnicity")} onClear={clearIssue} onChange={setEthnicity} />
                <Field name="birth_date" label="出生年月" type="month" value={birthDate} disabled={locked} issue={issueOf("birth_date")} onClear={clearIssue} onChange={setBirthDate} />
                <Field name="admin_title" label="行政职务" value={adminTitle} disabled={locked} issue={issueOf("admin_title")} onClear={clearIssue} onChange={setAdminTitle} />
                <Field name="professional_title" label="专业职务" value={professionalTitle} disabled={locked} issue={issueOf("professional_title")} onClear={clearIssue} onChange={setProfessionalTitle} />
                <Field name="research_expertise" label="研究专长" value={researchExpertise} disabled={locked} issue={issueOf("research_expertise")} onClear={clearIssue} onChange={setResearchExpertise} />
                <Field name="education" label="最后学历" value={education} disabled={locked} issue={issueOf("education")} onClear={clearIssue} onChange={setEducation} />
                <Field name="degree" label="最后学位" value={degree} disabled={locked} issue={issueOf("degree")} onClear={clearIssue} onChange={setDegree} />
                <Field name="mobile_phone" label="手机号码" value={mobilePhone} disabled={locked} issue={issueOf("mobile_phone")} onClear={clearIssue} onChange={setMobilePhone} />
              </div>
            </div>
            <RowEditor
              title="主要参加者（限 5 人）"
              hint="不含项目负责人，不包括科研管理、财务管理、后勤服务人员。出生年月可写成 1976-12 或 19761202。"
              columns={["姓名", "出生年月", "专业职务", "研究专长", "学历", "学位", "工作单位/部门"]}
              rows={members}
              keys={["name", "birth_date", "professional_title", "research_expertise", "education", "degree", "organization"]}
              disabled={locked}
              problem={(index, key) => issueOf(`members.${index}.${key}`)}
              onClear={(index, key) => clearIssue(`members.${index}.${key}`)}
              onChange={(index, key, value) => setMembers(members.map((row, rowIndex) => rowIndex === index ? { ...row, [key]: value } : row))}
              onAdd={() => members.length < 5 && setMembers([...members, blankMember()])}
              onRemove={(index) => setMembers(members.filter((_, rowIndex) => rowIndex !== index))}
            />
            <p className="sheet-hint">已录入 {members.filter((row) => row.name.trim()).length} / 5 人</p>
            {issueOf("members") ? <p className="text-[12px] text-cinnabar">{issueOf("members")}</p> : null}
            {detail.project_class !== "social" ? (
              <Area label="项目简介" hint="主要研究内容，主要技术经济等指标。" value={summary} disabled={locked} onChange={setSummary} />
            ) : null}
            <div className="rounded-lg border border-ink-200 bg-ink-50/40 p-3">
              <p className="text-[12px] font-medium text-ink-800">申请者承诺</p>
              <p className="mt-1.5 text-[11px] leading-5 text-ink-500">{commitmentText}</p>
              <p className="mt-2 text-[11px] text-ink-600">申请人（签章）：{detail.principal_name ?? "　　"}　·　提交时在预览页确认</p>
            </div>
          </Chapter>
        ) : null}
        {current?.id === "works" ? (
          <Chapter mark="二" title="负责人和项目组成员近五年来已开展的相关工作和取得的研究成果" hint="仅填写与本选题相关的成果。成果名称、研究者、成果形式及发表或交流情况须与佐证材料一致。">
            <RowEditor addLabel="新增成果" title="成果" columns={["成果名称", "研究者", "成果形式", "发表或交流情况（刊物、出版单位或学术交流层次）", "发表出版时间"]} rows={achievements} keys={["title", "researchers", "outcome_form", "publication", "published_on"]} disabled={locked} onChange={(index, key, value) => setAchievements(achievements.map((row, rowIndex) => rowIndex === index ? { ...row, [key]: value } : row))} onAdd={() => setAchievements([...achievements, { title: "", researchers: "", outcome_form: "", publication: "", published_on: "" }])} onRemove={(index) => setAchievements(achievements.filter((_, rowIndex) => rowIndex !== index))} />
            <p className="sheet-hint">共 {achievements.filter((row) => row.title.trim()).length} 项成果</p>
          </Chapter>
        ) : null}
        {current?.id === "design" ? (
          <Chapter mark="三" title="项目设计论证" aside="建议全文 5000–7000 字">
            <Area label="1. 选题依据" hint="国内外相关研究学术梳理及研究动态，本项目相对已有研究的独到学术价值和应用价值等。" value={design.topic_basis} disabled={locked} onChange={(value) => setDesign({ ...design, topic_basis: value })} />
            <Area label="2. 研究内容" hint="研究假设、核心概念、研究对象、总体框架、重点难点、主要目标等。" value={design.research_content} disabled={locked} onChange={(value) => setDesign({ ...design, research_content: value })} />
            <Area label="3. 思路方法" hint="本项目研究的具体研究方法、研究计划及其可行性等。" value={design.methodology} disabled={locked} onChange={(value) => setDesign({ ...design, methodology: value })} />
            <Area label="4. 创新之处" hint="在学术思想、学术观点、研究方法、破解难题等方面的特色和创新。" value={design.innovation} disabled={locked} onChange={(value) => setDesign({ ...design, innovation: value })} />
            <Area label="5. 预期成果" hint="成果形式、使用去向及预期社会效益等。" value={design.expected_outcome} disabled={locked} onChange={(value) => setDesign({ ...design, expected_outcome: value })} />
            <Area label="6. 参考文献" hint="开展本项目研究的主要中外参考文献。" value={design.reference_list} disabled={locked} onChange={(value) => setDesign({ ...design, reference_list: value })} />
          </Chapter>
        ) : null}
        {current?.id === "base" ? (
          <Chapter mark="四" title="研究基础和条件保障" aside="要求内容真实准确">
            <Area label="1. 学术简历" hint="项目负责人的主要学术经历，研究团队在选题相关研究领域的学术积累和贡献等。" value={foundation.academic_resume} disabled={locked} onChange={(value) => setFoundation({ ...foundation, academic_resume: value })} />
            <Area label="2. 研究基础" hint="前期相关代表性研究成果、核心观点及社会评价等；与本项目无关的成果不能作为前期成果填写。" value={foundation.research_basis} disabled={locked} onChange={(value) => setFoundation({ ...foundation, research_basis: value })} />
            <Area label="3. 承担项目" hint="包括项目名称、资助金额、结项情况、研究起止时间。申报书上是一张表，这里按同一段文字保存。" value={foundation.undertaken_projects} disabled={locked} onChange={(value) => setFoundation({ ...foundation, undertaken_projects: value })} />
            <Area label="4. 条件保障" hint="完成本项目研究的时间保证、资料设备等科研条件。" value={foundation.support_conditions} disabled={locked} onChange={(value) => setFoundation({ ...foundation, support_conditions: value })} />
          </Chapter>
        ) : null}
        {current?.id === "output" ? (
          <Chapter mark="五" title="预期研究成果" aside="阶段性成果限报 10 项，最终成果限报 2 项">
            <RowEditor addLabel="新增阶段性成果" title="主要阶段性成果（限报 10 项）" columns={["完成时间", "成果名称", "成果形式", "负责人"]} rows={stage} keys={["complete_on", "title", "outcome_form", "owner_name"]} disabled={locked} onChange={(index, key, value) => setStage(stage.map((row, rowIndex) => rowIndex === index ? { ...row, [key]: value } : row))} onAdd={() => stage.length < 10 && setStage([...stage, blankOutput()])} onRemove={(index) => setStage(stage.filter((_, rowIndex) => rowIndex !== index))} />
            <p className="sheet-hint">已填 {stage.filter((row) => row.title.trim()).length} / 10 项</p>
            <RowEditor addLabel="新增最终成果" title="最终研究成果（限报 2 项）" columns={["完成时间", "最终成果名称", "成果形式", "负责人"]} rows={finals} keys={["complete_on", "title", "outcome_form", "owner_name"]} disabled={locked} onChange={(index, key, value) => setFinals(finals.map((row, rowIndex) => rowIndex === index ? { ...row, [key]: value } : row))} onAdd={() => finals.length < 2 && setFinals([...finals, blankOutput()])} onRemove={(index) => setFinals(finals.filter((_, rowIndex) => rowIndex !== index))} />
            <p className="sheet-hint">已填 {finals.filter((row) => row.title.trim()).length} / 2 项</p>
          </Chapter>
        ) : null}
        {current?.id === "content" ? (
          <Chapter mark="二" title="主要研究内容" aside="自然科学类模板" hint="主要填写目标与任务需求分析，主要技术难点和问题分析，研究与开发任务与内容，关键技术、技术难点、创新点等">
            <textarea className="field-textarea" rows={14} value={researchContent} disabled={locked} aria-label="主要研究内容" onChange={(event) => setResearchContent(event.target.value)} />
          </Chapter>
        ) : null}
        {current?.id === "indicator" ? (
          <IndicatorSheet indicators={indicators} locked={locked} onIndicators={setIndicators} />
        ) : null}
        {current?.id === "plan" ? (
          <Chapter mark="四" title="计划进度及目标">
            <RowEditor addLabel="新增阶段" title="进度" columns={["开始日期", "结束日期", "阶段目标"]} rows={schedules} keys={["start_date", "end_date", "goal"]} disabled={locked} problem={(index, key) => issueOf(`schedules.${index}.${key}`)} onClear={(index, key) => clearIssue(`schedules.${index}.${key}`)} onChange={(index, key, value) => setSchedules(schedules.map((row, rowIndex) => rowIndex === index ? { ...row, [key]: value } : row))} onAdd={() => setSchedules([...schedules, { start_date: "", end_date: "", goal: "" }])} onRemove={(index) => setSchedules(schedules.filter((_, rowIndex) => rowIndex !== index))} />
          </Chapter>
        ) : null}
        {current?.id === "equip" ? (
          <Chapter mark="五" title="仪器设备购买清单（5000 元以上/台·套）" hint="仅填报单价 5000 元以上的仪器设备，金额＝数量 × 单价。">
            <RowEditor
              addLabel="新增设备"
              title="设备"
              columns={["仪器名称及规格型号", "数量", "单价（元）", "资金来源", "用途"]}
              extraHeader="金额（元）"
              extra={(row) => {
                const amount = Number(row.quantity || 0) * Number(row.unit_price || 0);
                return Number.isFinite(amount) ? amount.toFixed(2) : "0.00";
              }}
              rows={equipments}
              keys={["name_spec", "quantity", "unit_price", "funding_source", "purpose"]}
              disabled={locked}
              problem={(index, key) => issueOf(`equipments.${index}.${key}`)}
              onClear={(index, key) => clearIssue(`equipments.${index}.${key}`)}
              onChange={(index, key, value) => setEquipments(equipments.map((row, rowIndex) => rowIndex === index ? { ...row, [key]: value } : row))}
              onAdd={() => setEquipments([...equipments, { name_spec: "", quantity: "1", unit_price: "", funding_source: "", purpose: "" }])}
              onRemove={(index) => setEquipments(equipments.filter((_, rowIndex) => rowIndex !== index))}
            />
            <p className="text-right text-[12px] text-ink-600">合计（元） <span className="font-semibold text-teal-800">{equipments.reduce((sum, row) => sum + Number(row.quantity || 0) * Number(row.unit_price || 0), 0).toFixed(2)}</span></p>
          </Chapter>
        ) : null}
        {current?.id === "budget" ? (
          <Chapter mark="六" title="经费预算" aside="单位：万元" hint="申报书印刷稿金额栏写的是元。系统按万元保存，请按万元填写。绩效支出含在间接费用中，不重复计入合计。">
            <div className="overflow-x-auto rounded-lg border border-ink-200">
              <table className="sheet-table min-w-[640px]">
                <thead><tr><th className="w-12">序号</th><th>支出科目</th><th className="w-40">金额（万元）</th></tr></thead>
                <tbody>
                  {budget.map((line, index) => (
                    <tr key={line.code}>
                      <td className="text-center text-ink-400">{index + 1}</td>
                      <td className={line.fillable ? "" : "font-medium"}>{line.name}</td>
                      <td>
                        {line.fillable ? (
                          <>
                            <input className="field-input" value={line.amount} disabled={locked} aria-invalid={issueOf(`budget.${line.code}`) ? true : undefined} onChange={(event) => { clearIssue(`budget.${line.code}`); setBudget(budget.map((item) => item.code === line.code ? { ...item, amount: event.target.value } : item)); }} />
                            {issueOf(`budget.${line.code}`) ? <span className="mt-1 block text-[10.5px] text-cinnabar">{issueOf(`budget.${line.code}`)}</span> : null}
                          </>
                        ) : (
                          <span className="text-ink-400">{line.group_name === "total" ? (direct + indirect).toFixed(2) : line.group_name === "direct" ? direct.toFixed(2) : line.amount}</span>
                        )}
                      </td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
            <div className="flex flex-wrap items-center justify-end gap-4 text-[12px]">
              <span className="text-ink-500">直接费用小计 <span className="font-medium text-ink-800">{direct.toFixed(2)} 万元</span></span>
              <span className="rounded bg-teal-50 px-2.5 py-1 text-ink-700">经费合计 <span className="font-semibold text-teal-800">{(direct + indirect).toFixed(2)} 万元</span></span>
            </div>
          </Chapter>
        ) : null}
        {current?.id === "recommend" ? (
          <Chapter mark="七" title="推荐人意见" hint="负责人不具备中级及以上专业技术职务、且不具备研究生及以上学历时，须由两名具有高级专业技术职务的同行专家推荐。推荐人不能是负责人或参加者。">
            <div className="rounded-lg border border-gold-200 bg-gold-50 px-3 py-2 text-[12px] leading-5 text-gold-800">已具备中级及以上职称，或具备研究生及以上学历时，推荐人可以留空。姓名留空会删掉已保存的推荐人。</div>
            <div className="grid gap-3 sm:grid-cols-2">
              <Field name="name" label="推荐人姓名" value={recommender.name} disabled={locked} issue={issueOf("name")} onClear={clearIssue} onChange={(value) => setRecommender({ ...recommender, name: value })} />
              <Field name="professional_title" label="专业技术职务" value={recommender.professional_title} disabled={locked} issue={issueOf("professional_title")} onClear={clearIssue} onChange={(value) => setRecommender({ ...recommender, professional_title: value })} />
              <Field name="expertise" label="研究专长" value={recommender.expertise} disabled={locked} issue={issueOf("expertise")} onClear={clearIssue} onChange={(value) => setRecommender({ ...recommender, expertise: value })} />
              <Field name="organization" label="工作单位" value={recommender.organization} disabled={locked} issue={issueOf("organization")} onClear={clearIssue} onChange={(value) => setRecommender({ ...recommender, organization: value })} />
              <Field name="signed_on" label="签署日期" type="date" value={recommender.signed_on} disabled={locked} issue={issueOf("signed_on")} onClear={clearIssue} onChange={(value) => setRecommender({ ...recommender, signed_on: value })} />
            </div>
            <Area label="推荐意见" value={recommender.opinion} disabled={locked} onChange={(value) => setRecommender({ ...recommender, opinion: value })} />
          </Chapter>
        ) : null}
        {current?.id === "dept" ? (
          <Chapter mark="八" title="部门审核意见" aside="申报人只读">
            <p className="text-[12.5px] leading-6 text-ink-600">本栏由项目负责人所在部门填写审核意见、负责人签字和部门盖章。教师不能在这里写入意见。</p>
            <div className="rounded-lg border border-dashed border-ink-200 bg-ink-50/70 p-4 text-[12.5px] text-ink-500">
              当前状态：{statusLabels[detail.status] ?? detail.status}。意见在系部初审环节产生。
            </div>
          </Chapter>
        ) : null}
        {current?.id === "preview" ? (
          <Chapter mark="览" title="提交前确认">
            <p className="text-[13px] text-ink-700">项目名称：{title || "未填写"}</p>
            <p className="text-[13px] text-ink-700">经费合计：{(direct + indirect).toFixed(2)} 万元</p>
            <p className="text-[13px] text-ink-700">参加者：{members.filter((row) => row.name.trim()).length} 人</p>
            <div className="rounded-lg border border-ink-200 bg-ink-50/40 p-3 text-[11.5px] leading-5 text-ink-600">{commitmentText}</div>
            <label className="flex items-start gap-2 text-[12.5px] text-ink-700">
              <input type="checkbox" className="mt-0.5" checked={commitment} disabled={locked} onChange={(event) => setCommitment(event.target.checked)} />
              我确认填写内容真实，没有知识产权争议，同意以本申请作为有约束力的协议。
            </label>
            {detail.editable ? (
              <button type="button" disabled={busy} onClick={() => void submit()} className="inline-flex h-9 items-center gap-1 rounded-lg bg-teal-700 px-4 text-[12.5px] font-medium text-white disabled:opacity-50">
                <Icon name="mdi:send-outline" />确认提交
              </button>
            ) : <p className="text-[12px] text-ink-500">这份申报书当前不能修改。</p>}
          </Chapter>
        ) : null}
      </div>

      {current?.id !== "preview" ? (
        <div className="mt-4 flex flex-wrap items-center justify-between gap-2 rounded-xl border border-ink-200/80 bg-white px-4 py-3">
          <button type="button" disabled={step === 0 || busy} onClick={() => void jump(step - 1)} className="inline-flex h-8 items-center gap-1 rounded-lg border border-ink-200 px-3 text-[11.5px] disabled:opacity-40"><Icon name="mdi:arrow-left" className="text-sm" />上一步</button>
          <div className="flex items-center gap-2">
            {issues.length > 0 ? <span className="text-[12px] text-cinnabar">请按上面列出的位置修改</span> : null}
            {detail.editable && current?.id !== "dept" ? <button type="button" disabled={busy} onClick={() => void persist(false)} className="inline-flex h-8 items-center gap-1 rounded-lg border border-ink-200 px-3 text-[11.5px]"><Icon name="mdi:content-save-outline" className="text-sm" />保存草稿</button> : null}
            <button type="button" disabled={busy || step >= steps.length - 1} onClick={() => void persist(true)} className="inline-flex h-8 items-center gap-1 rounded-lg bg-teal-700 px-3.5 text-[11.5px] font-medium text-white disabled:opacity-40">下一步{steps[step + 1] ? `：${steps[step + 1]?.title}` : ""}<Icon name="mdi:arrow-right" className="text-sm" /></button>
          </div>
        </div>
      ) : null}
    </div>
  );
}

function fillOutput(row: { complete_on?: string | null; title: string; outcome_form?: string | null; owner_name?: string | null }): OutputRow {
  return { complete_on: row.complete_on ?? "", title: row.title, outcome_form: row.outcome_form ?? "", owner_name: row.owner_name ?? "" };
}

function blankOutput(): OutputRow {
  return { complete_on: "", title: "", outcome_form: "", owner_name: "" };
}

function sumBudget(lines: ApplicationDetail["budget"]["lines"], group: string) {
  return lines.filter((line) => line.group_name === group && line.fillable).reduce((sum, line) => sum + Number(line.amount || 0), 0);
}

function CategoryCard({ icon, tone, title, text, chips, action, onClick, disabled }: { icon: string; tone: "gold" | "teal"; title: string; text: string; chips: string[]; action: string; onClick: () => void; disabled: boolean }) {
  return (
    <button type="button" disabled={disabled} onClick={onClick} className="rounded-xl border border-ink-200 bg-white p-4 text-left transition hover:border-teal-300 disabled:opacity-50">
      <span className={`flex h-10 w-10 items-center justify-center rounded-lg ${tone === "gold" ? "bg-gold-50 text-gold-600" : "bg-teal-50 text-teal-600"}`}><Icon name={icon} className="text-xl" /></span>
      <h3 className="mt-3 text-[13.5px] font-semibold text-ink-900">{title}</h3>
      <p className="mt-1 text-[11px] leading-5 text-ink-500">{text}</p>
      <div className="mt-2 flex flex-wrap gap-1.5">{chips.map((chip) => <span key={chip} className="rounded bg-ink-50 px-1.5 py-0.5 text-[10px] text-ink-500">{chip}</span>)}</div>
      <div className="mt-2.5 flex items-center justify-between border-t border-ink-100 pt-2.5 text-[11px]">
        <span className="text-[10px] text-ink-400">{action}</span>
        <span className="inline-flex items-center gap-1 font-medium text-teal-700">选择并进入<Icon name="mdi:arrow-right" className="text-sm" /></span>
      </div>
    </button>
  );
}

function Chapter({ mark, title, hint, aside, action, children }: { mark: string; title: string; hint?: string; aside?: string; action?: ReactNode; children: ReactNode }) {
  return (
    <section className="rounded-xl border border-ink-200/80 bg-white p-4 shadow-sm sm:p-5">
      <div className="flex flex-wrap items-center gap-2">
        <span className="flex h-6 w-6 items-center justify-center rounded bg-ink-900 text-[10px] font-semibold text-white">{mark}</span>
        <h2 className="text-[14px] font-semibold text-ink-900">{title}</h2>
        {aside ? <span className="ml-auto text-[10.5px] text-ink-400">{aside}</span> : null}
        {action ? <span className={aside ? "" : "ml-auto"}>{action}</span> : null}
      </div>
      {hint ? <p className="sheet-hint mt-1.5">{hint}</p> : null}
      <div className="mt-3.5 space-y-4">{children}</div>
    </section>
  );
}

function Field({ name, label, value, onChange, onClear, issue, disabled, type = "text" }: { name?: string; label: string; value: string; onChange: (value: string) => void; onClear?: (field: string) => void; issue?: string; disabled?: boolean; type?: string }) {
  return (
    <label className="block">
      <span className="field-label">{label}</span>
      <input className="field-input" type={type} value={value} disabled={disabled} aria-invalid={issue ? true : undefined} onChange={(event) => { if (name) onClear?.(name); onChange(event.target.value); }} />
      {issue ? <span className="mt-1 block text-[11px] text-cinnabar">{issue}</span> : null}
    </label>
  );
}

function ReadOnly({ label, value }: { label: string; value: string }) {
  return <label className="block"><span className="field-label">{label}</span><input className="field-input bg-ink-50 text-ink-600" value={value} readOnly /></label>;
}

function Area({ label, hint, value, onChange, disabled }: { label: string; hint?: string; value: string; onChange: (value: string) => void; disabled?: boolean }) {
  return (
    <label className="block">
      <span className="field-label">{label}</span>
      {hint ? <p className="sheet-hint mb-1.5">{hint}</p> : null}
      <textarea className="field-textarea" rows={5} value={value} disabled={disabled} onChange={(event) => onChange(event.target.value)} />
    </label>
  );
}

function RowEditor<T extends object>({ title, hint, addLabel = "新增", columns, rows, keys, disabled, problem, onClear, onChange, onAdd, onRemove, extraHeader, extra }: { title: string; hint?: string; addLabel?: string; columns: string[]; rows: T[]; keys: string[]; disabled?: boolean; problem?: (index: number, key: string) => string | undefined; onClear?: (index: number, key: string) => void; onChange: (index: number, key: string, value: string) => void; onAdd: () => void; onRemove: (index: number) => void; extraHeader?: string; extra?: (row: T) => string }) {
  return (
    <div>
      <div className="flex flex-wrap items-center gap-2">
        <p className="text-[12.5px] font-semibold text-ink-800">{title}</p>
        {hint ? <span className="sheet-hint">{hint}</span> : null}
        <button type="button" disabled={disabled} onClick={onAdd} className="ml-auto inline-flex h-7 items-center gap-1 rounded-md border border-teal-200 bg-teal-50 px-2.5 text-[10.5px] font-medium text-teal-700 disabled:opacity-40"><Icon name="mdi:plus" className="text-sm" />{addLabel}</button>
      </div>
      <div className="mt-2 overflow-x-auto rounded-lg border border-ink-200">
        <table className="sheet-table min-w-[760px]">
          <thead><tr><th className="w-12">序号</th>{columns.map((column) => <th key={column}>{column}</th>)}{extraHeader ? <th className="w-28">{extraHeader}</th> : null}<th className="w-14">操作</th></tr></thead>
          <tbody>
            {rows.length === 0 ? <tr><td colSpan={columns.length + (extraHeader ? 3 : 2)} className="px-3 py-4 text-center text-ink-400">还没有记录，点右上角新增</td></tr> : null}
            {rows.map((row, index) => (
              <tr key={`${title}-${index}`}>
                <td className="text-center text-ink-400">{index + 1}</td>
                {keys.map((key) => {
                  const issue = problem?.(index, key);
                  return (
                    <td key={key}>
                      <input value={String((row as Record<string, string>)[key] ?? "")} disabled={disabled} aria-invalid={issue ? true : undefined} placeholder={key === "birth_date" ? "1976-12" : undefined} onChange={(event) => { onClear?.(index, key); onChange(index, key, event.target.value); }} />
                      {issue ? <span className="mt-1 block text-[10.5px] text-cinnabar">{issue}</span> : null}
                    </td>
                  );
                })}
                {extra ? <td className="text-ink-600">{extra(row)}</td> : null}
                <td><button type="button" disabled={disabled} onClick={() => onRemove(index)} className="text-red-600 disabled:opacity-40" aria-label="删除行"><Icon name="mdi:delete-outline" /></button></td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </div>
  );
}

function plainContent(text: string) {
  return text
    .replaceAll("【研究内容】", "")
    .replaceAll("【拟解决的关键科学问题】", "")
    .replaceAll("【研究方案与技术路线】", "")
    .replace(/\n{3,}/g, "\n\n")
    .trim();
}

function IndicatorSheet({ indicators, locked, onIndicators }: {
  indicators: Record<string, number | string | null>;
  locked: boolean;
  onIndicators: (value: Record<string, number | string | null> | ((current: Record<string, number | string | null>) => Record<string, number | string | null>)) => void;
}) {
  function box(key: string, label: string, money = false) {
    const stored = indicators[key];
    const blank = stored === null || stored === undefined || stored === "" || Number(stored) === 0;
    return (
      <input
        aria-label={label}
        type="number"
        min={0}
        step={money ? "0.01" : "1"}
        disabled={locked}
        placeholder={money ? "0.00" : "0"}
        value={blank ? "" : stored}
        onFocus={(event) => event.currentTarget.select()}
        onChange={(event) => {
          const raw = event.target.value;
          if (raw === "") {
            onIndicators((current) => ({ ...current, [key]: money ? "0" : 0 }));
            return;
          }
          const next = money ? raw.replace(/^0+(?=\d)/, "") : Number(raw);
          if (!money && Number.isNaN(next)) return;
          onIndicators((current) => ({ ...current, [key]: next }));
        }}
      />
    );
  }

  return (
    <Chapter mark="三" title="主要技术经济指标" hint="按申报书填写。数量为非负整数，累计新增销售收入和利税的单位是万元。">
      <div className="overflow-x-auto">
        <table className="metric-sheet">
          <tbody>
            <tr><th colSpan={6} className="band">（一）量化考核指标</th></tr>
            <tr><th colSpan={6} className="band">1.科技产出</th></tr>
            <tr>
              <th rowSpan={3} className="side">论文（篇）</th>
              <th>期刊</th>
              <th>国际</th>
              <th>国内</th>
              <th colSpan={2}>省级</th>
            </tr>
            <tr>
              <th>核心期刊</th>
              <td>{box("paper_core", "核心期刊（国际）")}</td>
              <td>{box("paper_domestic", "核心期刊（国内）")}</td>
              <td colSpan={2} className="blocked" title="省级不设核心期刊" />
            </tr>
            <tr>
              <th>一般期刊</th>
              <td>{box("paper_general", "一般期刊（国际）")}</td>
              <td>{box("paper_international", "一般期刊（国内）")}</td>
              <td colSpan={2}>{box("paper_provincial", "一般期刊（省级）")}</td>
            </tr>
            <tr>
              <th rowSpan={2} className="side">报告（篇）</th>
              <th>科技报告</th>
              <th>研究报告</th>
              <th colSpan={2}>科技成果转化</th>
              <th>其他报告</th>
            </tr>
            <tr>
              <td>{box("report_tech", "科技报告")}</td>
              <td>{box("report_research", "研究报告")}</td>
              <td colSpan={2}>{box("report_transfer", "科技成果转化")}</td>
              <td>{box("report_other", "其他报告")}</td>
            </tr>
            <tr>
              <th className="side">专著（部）</th>
              <td colSpan={5}>{box("monograph", "专著")}</td>
            </tr>
            <tr>
              <th rowSpan={6} className="side">知识产权（件）</th>
              <th>专利</th>
              <th>发明</th>
              <th>实用新型</th>
              <th>外观设计</th>
              <th>PCT</th>
            </tr>
            <tr>
              <th>申请（受理）</th>
              <td>{box("patent_invention_applied", "发明专利申请")}</td>
              <td>{box("patent_utility_applied", "实用新型申请")}</td>
              <td>{box("patent_design_applied", "外观设计申请")}</td>
              <td>{box("patent_pct_applied", "PCT申请")}</td>
            </tr>
            <tr>
              <th>授权数</th>
              <td>{box("patent_invention_granted", "发明专利授权")}</td>
              <td>{box("patent_utility_granted", "实用新型授权")}</td>
              <td>{box("patent_design_granted", "外观设计授权")}</td>
              <td>{box("patent_pct_granted", "PCT授权")}</td>
            </tr>
            <tr>
              <th>其他</th>
              <th>软件著作</th>
              <th>集成电路布图设</th>
              <th colSpan={2}>植物新品种</th>
            </tr>
            <tr>
              <th>申请数</th>
              <td>{box("software_copyright_applied", "软件著作权申请")}</td>
              <td>{box("ic_layout_applied", "集成电路布图申请")}</td>
              <td colSpan={2}>{box("plant_variety_applied", "植物新品种申请")}</td>
            </tr>
            <tr>
              <th>登记数</th>
              <td>{box("software_copyright_registered", "软件著作权登记")}</td>
              <td>{box("ic_layout_registered", "集成电路布图登记")}</td>
              <td colSpan={2}>{box("plant_variety_registered", "植物新品种登记")}</td>
            </tr>
            <tr>
              <th rowSpan={3} className="side">标准（项）</th>
              <th>标准分类</th>
              <th>国家标准</th>
              <th>行业标准</th>
              <th>地方标准</th>
              <th>企业标准</th>
            </tr>
            <tr>
              <th>牵头</th>
              <td>{box("standard_national_lead", "国家标准牵头")}</td>
              <td>{box("standard_industry_lead", "行业标准牵头")}</td>
              <td>{box("standard_local_lead", "地方标准牵头")}</td>
              <td>{box("standard_enterprise_lead", "企业标准牵头")}</td>
            </tr>
            <tr>
              <th>参与</th>
              <td>{box("standard_national_participate", "国家标准参与")}</td>
              <td>{box("standard_industry_participate", "行业标准参与")}</td>
              <td>{box("standard_local_participate", "地方标准参与")}</td>
              <td>{box("standard_enterprise_participate", "企业标准参与")}</td>
            </tr>
            <tr>
              <th className="side">中试数（条）</th>
              <td colSpan={5}>{box("pilot_line", "中试数")}</td>
            </tr>
            <tr>
              <th className="side">生产线（条）</th>
              <td colSpan={5}>{box("production_line", "生产线")}</td>
            </tr>
            <tr>
              <th className="side">累计新增销售收入（万元）</th>
              <td colSpan={5}>{box("new_sales_income", "累计新增销售收入", true)}</td>
            </tr>
            <tr>
              <th className="side">累计新增利税（万元）</th>
              <td colSpan={5}>{box("new_profit_tax", "累计新增利税", true)}</td>
            </tr>
            <tr><th colSpan={6} className="band">2.研发服务能力</th></tr>
            <tr>
              <th rowSpan={2} className="side">培养、引进人才（人）</th>
              <th colSpan={2}>培养高级职称以上</th>
              <th>培养中级职称以上</th>
              <th colSpan={2}>引进高级职称以上</th>
            </tr>
            <tr>
              <td colSpan={2}>{box("talent_train_senior", "培养高级职称以上")}</td>
              <td>{box("talent_train_intermediate", "培养中级职称以上")}</td>
              <td colSpan={2}>{box("talent_introduce_senior", "引进高级职称以上")}</td>
            </tr>
            <tr>
              <th rowSpan={2} className="side">引进新产品、新技术、新材料情况</th>
              <th>新产品（种）</th>
              <th>新技术（项）</th>
              <th>新工艺（项）</th>
              <th colSpan={2}>新材料（项）</th>
            </tr>
            <tr>
              <td>{box("new_product", "新产品")}</td>
              <td>{box("new_technology", "新技术")}</td>
              <td>{box("new_process", "新工艺")}</td>
              <td colSpan={2}>{box("new_material", "新材料")}</td>
            </tr>
            <tr>
              <th rowSpan={2} className="side">其他指标</th>
              <th colSpan={2}>支撑平台（个）</th>
              <th>试验基地（个）</th>
              <th colSpan={2}>示范点（区）（个）</th>
            </tr>
            <tr>
              <td colSpan={2}>{box("support_platform", "支撑平台")}</td>
              <td>{box("test_base", "试验基地")}</td>
              <td colSpan={2}>{box("demo_site", "示范点")}</td>
            </tr>
            <tr><th colSpan={6} className="band">（二）主要技术经济指标及社会效益说明</th></tr>
            <tr>
              <td colSpan={6}>
                <textarea
                  aria-label="主要技术经济指标及社会效益说明"
                  rows={8}
                  disabled={locked}
                  value={String(indicators.benefit_note ?? "")}
                  onChange={(event) => onIndicators((current) => ({ ...current, benefit_note: event.target.value }))}
                />
              </td>
            </tr>
          </tbody>
        </table>
      </div>
    </Chapter>
  );
}

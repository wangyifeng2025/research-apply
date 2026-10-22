import type { FieldIssue } from "@/lib/api";
import type { MemberInput } from "@/lib/applications";

const dateHint = "请写成 1976-12、1976-12-02，或纯数字 197612、19761202";

export function looseDate(raw: string): string | null {
  const value = raw.trim();
  if (!value) return null;
  const dashed = value.replaceAll("/", "-").replaceAll(".", "-");
  if (/^\d{4}-\d{2}-\d{2}$/.test(dashed) && calendarDate(dashed)) return null;
  if (/^\d{4}-\d{2}$/.test(dashed) && calendarDate(`${dashed}-01`)) return null;
  if (/^\d{8}$/.test(value) && calendarDate(`${value.slice(0, 4)}-${value.slice(4, 6)}-${value.slice(6, 8)}`)) return null;
  if (/^\d{6}$/.test(value) && calendarDate(`${value.slice(0, 4)}-${value.slice(4, 6)}-01`)) return null;
  return dateHint;
}

function calendarDate(value: string): boolean {
  const [year, month, day] = value.split("-").map(Number);
  if (!year || !month || !day) return false;
  const date = new Date(Date.UTC(year, month - 1, day));
  return date.getUTCFullYear() === year && date.getUTCMonth() === month - 1 && date.getUTCDate() === day;
}

function tooLong(value: string, max: number): boolean {
  return [...value.trim()].length > max;
}

function push(issues: FieldIssue[], field: string, message: string) {
  issues.push({ field, message });
}

export function checkBasicSection(input: {
  title: string;
  keywords: string;
  filledOn: string;
  birthDate: string;
  gender: string;
  ethnicity: string;
  adminTitle: string;
  professionalTitle: string;
  researchExpertise: string;
  education: string;
  degree: string;
  mobilePhone: string;
  members: MemberInput[];
}): FieldIssue[] {
  const issues: FieldIssue[] = [];
  if (tooLong(input.title, 40)) push(issues, "title", "项目名称最多 40 个字");
  const keywords = input.keywords.split(/[;；]/).map((item) => item.trim()).filter(Boolean);
  if (keywords.length > 5) push(issues, "keywords", "关键词最多 5 个，请用分号隔开");
  const filled = looseDate(input.filledOn);
  if (filled) push(issues, "filled_on", `填表日期${filled}`);
  const birth = looseDate(input.birthDate);
  if (birth) push(issues, "birth_date", `出生年月${birth}`);
  if (tooLong(input.gender, 8)) push(issues, "gender", "性别最多 8 个字");
  if (tooLong(input.ethnicity, 32)) push(issues, "ethnicity", "民族最多 32 个字");
  if (tooLong(input.adminTitle, 64)) push(issues, "admin_title", "行政职务最多 64 个字");
  if (tooLong(input.professionalTitle, 64)) push(issues, "professional_title", "专业职务最多 64 个字");
  if (tooLong(input.researchExpertise, 120)) push(issues, "research_expertise", "研究专长最多 120 个字");
  if (tooLong(input.education, 32)) push(issues, "education", "最后学历最多 32 个字");
  if (tooLong(input.degree, 32)) push(issues, "degree", "最后学位最多 32 个字");
  if (tooLong(input.mobilePhone, 20)) push(issues, "mobile_phone", "手机号码最多 20 位");

  let filledRows = 0;
  input.members.forEach((row, index) => {
    const blank = [row.name, row.birth_date, row.professional_title, row.research_expertise, row.education, row.degree, row.organization].every((item) => !item.trim());
    if (blank) return;
    filledRows += 1;
    const label = row.name.trim() ? `第 ${index + 1} 位参加者「${row.name.trim()}」` : `第 ${index + 1} 位参加者`;
    if (!row.name.trim()) push(issues, `members.${index}.name`, `${label}写了其他资料，但没有填写姓名`);
    else if (tooLong(row.name, 64)) push(issues, `members.${index}.name`, `${label}的姓名最多 64 个字`);
    const memberBirth = looseDate(row.birth_date);
    if (memberBirth) push(issues, `members.${index}.birth_date`, `${label}的出生年月${memberBirth}`);
    if (tooLong(row.professional_title, 64)) push(issues, `members.${index}.professional_title`, `${label}的专业职务最多 64 个字`);
    if (tooLong(row.research_expertise, 120)) push(issues, `members.${index}.research_expertise`, `${label}的研究专长最多 120 个字`);
    if (tooLong(row.education, 32)) push(issues, `members.${index}.education`, `${label}的学历最多 32 个字`);
    if (tooLong(row.degree, 32)) push(issues, `members.${index}.degree`, `${label}的学位最多 32 个字`);
    if (tooLong(row.organization, 120)) push(issues, `members.${index}.organization`, `${label}的工作单位最多 120 个字`);
  });
  if (filledRows > 5) push(issues, "members", "主要参加者最多 5 人，不含项目负责人");
  return issues;
}

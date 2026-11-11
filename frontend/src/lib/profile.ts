import { apiSend } from "@/lib/api";

export type Profile = {
  id: string;
  username: string;
  real_name: string;
  employee_no: string | null;
  gender: string | null;
  ethnicity: string | null;
  birth_date: string | null;
  admin_title: string | null;
  professional_title: string | null;
  research_expertise: string | null;
  education: string | null;
  degree: string | null;
  phone: string | null;
  email: string | null;
  department_name: string | null;
};

export type ProfileInput = {
  gender: string;
  ethnicity: string;
  birth_date: string;
  admin_title: string;
  professional_title: string;
  research_expertise: string;
  education: string;
  degree: string;
  phone: string;
  email: string;
};

export function fetchProfile(): Promise<Profile> {
  return apiSend("/api/profile");
}

export function updateProfile(input: ProfileInput): Promise<Profile> {
  return apiSend("/api/profile", { method: "PATCH", body: JSON.stringify(input) });
}

export function profileInput(profile: Profile): ProfileInput {
  return {
    gender: profile.gender ?? "",
    ethnicity: profile.ethnicity ?? "",
    birth_date: profile.birth_date ?? "",
    admin_title: profile.admin_title ?? "",
    professional_title: profile.professional_title ?? "",
    research_expertise: profile.research_expertise ?? "",
    education: profile.education ?? "",
    degree: profile.degree ?? "",
    phone: profile.phone ?? "",
    email: profile.email ?? "",
  };
}

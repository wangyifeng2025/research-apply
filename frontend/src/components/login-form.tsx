"use client";

import { zodResolver } from "@hookform/resolvers/zod";
import { useRouter } from "next/navigation";
import { useEffect, useState } from "react";
import { useForm, useWatch } from "react-hook-form";

import { Button } from "@/components/ui/button";
import { Checkbox } from "@/components/ui/checkbox";
import { Input } from "@/components/ui/input";
import { ApiError, loginRequest } from "@/lib/api";
import {
  backendRoles,
  loginSchema,
  loginStorageKey,
  readStoredLogin,
  roleHomes,
  rolePresets,
  type LoginRole,
  type LoginValues,
} from "@/lib/login";

const roleOrder: LoginRole[] = ["teacher", "department", "office", "expert", "admin"];

const steps = [
  "教师在线填报",
  "系审核",
  "科研处审核",
  "专家盲审",
];

type LoginStatus = {
  cycleTitle: string;
  cycleStatus: string | null;
  budgetSubjectCount: number | null;
  database: string;
  connected: boolean;
};

export function LoginForm({
  cycleTitle,
  cycleStatus,
  budgetSubjectCount,
  database,
  connected,
}: LoginStatus) {
  const router = useRouter();
  const [showPassword, setShowPassword] = useState(false);
  const [notice, setNotice] = useState<string | null>(null);
  const [submitError, setSubmitError] = useState<string | null>(null);
  const form = useForm<LoginValues>({
    resolver: zodResolver(loginSchema),
    defaultValues: {
      role: "teacher",
      username: "",
      password: "",
      remember: true,
    },
  });

  const role = useWatch({ control: form.control, name: "role" });
  const remember = useWatch({ control: form.control, name: "remember" });
  const preset = rolePresets[role];
  const {
    formState: { errors, isSubmitting },
  } = form;

  useEffect(() => {
    const stored = readStoredLogin();
    if (!stored) {
      return;
    }
    form.reset({
      role: stored.role,
      username: stored.username,
      password: "",
      remember: true,
    });
  }, [form]);

  function chooseRole(next: LoginRole) {
    form.setValue("role", next, { shouldValidate: form.formState.isSubmitted });
    form.setValue("username", "", { shouldValidate: form.formState.isSubmitted });
    form.setValue("password", "", { shouldValidate: form.formState.isSubmitted });
    setNotice(null);
    setSubmitError(null);
  }

  async function onSubmit(values: LoginValues) {
    setSubmitError(null);
    try {
      const user = await loginRequest(values.username, values.password);
      const expected = backendRoles[values.role];
      if (!user.roles.includes(expected)) {
        setSubmitError(`账号 ${user.real_name} 没有「${rolePresets[values.role].label}」权限`);
        return;
      }
      const profile = {
        v: 1,
        username: values.username,
        role: values.role,
        realName: user.real_name,
        departmentName: user.department_name,
      };
      sessionStorage.setItem(loginStorageKey, JSON.stringify(profile));
      if (values.remember) {
        localStorage.setItem(loginStorageKey, JSON.stringify(profile));
      } else {
        localStorage.removeItem(loginStorageKey);
      }
      router.push(roleHomes[values.role]);
    } catch (error) {
      setSubmitError(error instanceof ApiError ? error.message : "登录失败");
    }
  }

  const connectionLabel = connected ? "已连接" : "未连接，请先启动后端";

  return (
    <div className="fixed inset-0 z-10 h-dvh overflow-hidden bg-ink-50 text-ink-800 antialiased">
      <div className="flex h-full flex-col lg:flex-row">
        <section className="relative flex shrink-0 flex-col justify-between overflow-hidden bg-ink-950 px-6 py-4 text-white lg:h-full lg:w-[54%] lg:px-10 lg:py-6">
          <div className="pointer-events-none absolute inset-0 bg-[linear-gradient(rgba(255,255,255,0.045)_1px,transparent_1px),linear-gradient(90deg,rgba(255,255,255,0.045)_1px,transparent_1px)] bg-size-[46px_46px]" />
          <div className="pointer-events-none absolute -top-24 -right-32 h-105 w-105 rounded-full bg-teal-500/20 blur-[110px]" />
          <div className="pointer-events-none absolute -bottom-40 left-10 h-105 w-105 rounded-full bg-gold-500/10 blur-[120px]" />

          <div className="relative z-10 flex items-center gap-3">
            <div className="flex h-10 w-10 items-center justify-center rounded-xl bg-linear-to-br from-teal-400 to-teal-700 text-lg font-semibold shadow-lg shadow-teal-900/40">
              研
            </div>
            <div className="leading-tight">
              <div className="text-[16px] font-semibold tracking-wide">院级科研项目申报</div>
              <div className="mt-0.5 text-[11px] tracking-[0.18em] text-ink-300">
                黔西南民族职业技术学院 · 科研处
              </div>
            </div>
          </div>

          <div className="relative z-10 mt-4 hidden max-w-155 lg:mt-0 lg:block">
            <div className="mb-3 inline-flex items-center gap-2 rounded-full border border-teal-400/30 bg-teal-400/10 px-3 py-1 text-[11px] font-medium text-teal-200">
              教师、系、科研处、专家四级留痕
            </div>
            <h1 className="text-[30px] leading-[1.3] font-semibold tracking-tight">
              让每一份科研申报
              <br />
              <span className="text-teal-300">流转清晰、有据可查</span>
            </h1>
            <p className="mt-3 max-w-120 text-[12px] leading-6 text-ink-300">
              面向社会科学与自然科学两类项目。教师在线填写申报书，经系审核、科研处审核后进入专家盲审。
            </p>
            <div className="mt-5 grid max-w-140 grid-cols-4 gap-2">
              {steps.map((step, index) => (
                <div key={step} className="rounded-xl border border-white/10 bg-white/4 px-2.5 py-2">
                  <div className="text-[10px] font-semibold text-teal-200">0{index + 1}</div>
                  <div className="mt-1 text-[12px] text-ink-200">{step}</div>
                </div>
              ))}
            </div>
          </div>

          <div className="relative z-10 mt-4 grid gap-2 lg:mt-0">
            <div className="rounded-xl border border-white/10 bg-white/5 px-4 py-3">
              <p className="text-[11px] tracking-[0.18em] text-ink-300">本批次</p>
              <p className="mt-1 text-[15px] leading-snug font-medium">{cycleTitle}</p>
              <p className="mt-1 text-[12px] text-ink-300">
                {cycleStatus ?? "等待数据库"}
                {budgetSubjectCount === null ? "" : ` · 经费科目 ${budgetSubjectCount} 项`}
              </p>
            </div>
            <div className="flex items-center gap-2 px-1 text-[12px] text-ink-200">
              <span className={`h-1.5 w-1.5 rounded-full ${connected ? "bg-teal-400" : "bg-gold-400"}`} />
              数据库 {database} {connectionLabel}
            </div>
          </div>
        </section>

        <section className="flex min-h-0 flex-1 items-center justify-center px-5 py-3 sm:px-8">
          <div className="w-full max-w-110">
            <div className="mb-3">
              <div className="mb-1 flex items-center gap-2 text-[11px] font-medium text-teal-700">
                <span className="h-px w-5 bg-teal-600" />
                账号登录
              </div>
              <h2 className="text-[22px] font-semibold tracking-tight text-ink-900">欢迎登录</h2>
              <p className="mt-1 text-[12px] leading-5 text-ink-500">
                选择角色进入对应工作台。账号和密码由科研申报服务校验。
              </p>
            </div>

            <div>
              <div className="mb-1.5 flex items-center justify-between">
                <span className="text-[12px] font-medium text-ink-700">登录角色</span>
                <span className="text-[11px] text-ink-400">共 5 类角色</span>
              </div>
              <div className="grid grid-cols-2 gap-1.5">
                {roleOrder.map((item) => {
                  const current = rolePresets[item];
                  const active = role === item;
                  return (
                    <Button
                      key={item}
                      type="button"
                      variant="outline"
                      aria-pressed={active}
                      onClick={() => chooseRole(item)}
                      className={`h-auto justify-start gap-2 px-2.5 py-1.5 text-left font-normal whitespace-normal ${
                        item === "admin" ? "col-span-2" : ""
                      } ${
                        active
                          ? "border-teal-500 bg-teal-50 shadow-[inset_0_0_0_1px_#218b84]"
                          : "hover:border-teal-300 hover:bg-teal-50/50"
                      }`}
                    >
                      <span
                        className={`flex h-7 w-7 shrink-0 items-center justify-center rounded-md text-[12px] font-semibold ${
                          active ? "bg-teal-500 text-white" : "bg-ink-50 text-ink-500"
                        }`}
                      >
                        {current.mark}
                      </span>
                      <span className="min-w-0">
                        <span className={`block text-[13px] font-medium ${active ? "text-teal-800" : "text-ink-800"}`}>
                          {current.label}
                        </span>
                        <span className="block truncate text-[11px] text-ink-400">{current.summary}</span>
                      </span>
                    </Button>
                  );
                })}
              </div>
              {errors.role ? <p className="mt-2 text-[12px] text-red-700">{errors.role.message}</p> : null}
            </div>

            <form className="mt-3 space-y-2" onSubmit={form.handleSubmit(onSubmit)} noValidate>
              <input type="hidden" {...form.register("role")} />
              <div>
                <label className="mb-1 block text-[12px] font-medium text-ink-700" htmlFor="username">
                  账号
                </label>
                <Input
                  id="username"
                  autoComplete="username"
                  aria-invalid={errors.username ? true : undefined}
                  placeholder="请输入工号或账号"
                  className="h-9"
                  {...form.register("username")}
                />
                {errors.username ? (
                  <p className="mt-1.5 text-[12px] text-red-700">{errors.username.message}</p>
                ) : null}
              </div>

              <div>
                <label className="mb-1 block text-[12px] font-medium text-ink-700" htmlFor="password">
                  密码
                </label>
                <div className="relative">
                  <Input
                    id="password"
                    type={showPassword ? "text" : "password"}
                    autoComplete="current-password"
                    aria-invalid={errors.password ? true : undefined}
                    placeholder="请输入登录密码"
                    className="h-9 pr-16"
                    {...form.register("password")}
                  />
                  <Button
                    type="button"
                    variant="ghost"
                    size="sm"
                    className="absolute top-1/2 right-1 -translate-y-1/2"
                    onClick={() => setShowPassword((value) => !value)}
                  >
                    {showPassword ? "隐藏" : "显示"}
                  </Button>
                </div>
                {errors.password ? (
                  <p className="mt-1.5 text-[12px] text-red-700">{errors.password.message}</p>
                ) : null}
              </div>

              <div className="flex items-center justify-between pt-0.5">
                <label className="flex cursor-pointer items-center gap-2 text-[12px] text-ink-600">
                  <Checkbox
                    checked={remember}
                    onCheckedChange={(value) => form.setValue("remember", value === true)}
                  />
                  记住账号
                </label>
                <Button
                  type="button"
                  variant="subtle"
                  className="h-auto px-0 hover:underline"
                  onClick={() => setNotice("请联系科研处重置密码。")}
                >
                  忘记密码？
                </Button>
              </div>

              {notice ? <p className="text-[12px] text-ink-600">{notice}</p> : null}
              {submitError ? <p className="text-[12px] text-red-700">{submitError}</p> : null}

              <Button type="submit" size="lg" disabled={isSubmitting} className="w-full">
                {isSubmitting ? "正在验证身份…" : "登录科研申报系统"}
              </Button>
            </form>

            <div className="mt-2 rounded-lg border border-ink-100 bg-ink-50/70 px-3 py-2 text-[11px] leading-5 text-ink-500">
              <span className="font-medium text-ink-700">当前角色：</span>
              {preset.label}。{preset.note}
              <span className="mt-0.5 block text-ink-400">账号和密码由科研处分配。选错角色时不会进入该工作台。</span>
            </div>
          </div>
        </section>
      </div>
    </div>
  );
}

# 院级科研项目申报系统：开发指南

## 项目概览

本项目是黔西南民族职业技术学院科研处的院级科研项目在线申报与审核系统。目标业务流程为：教师填报并提交，系审核，科研处审核，专家匿名盲审，科研处决定立项或不予立项。

项目类别使用以下稳定代码：

- `social`：B. 社会科学类，使用社会科学申报书；
- `natural`：A. 自然科学类；
- `dual_high`：D. “双高”校建设专项，使用自然科学申报书。

账号通过 `user_roles` 支持多角色：`teacher`、`department_reviewer`、`research_office`、`expert`、`admin`。后端 `require_role` 允许管理员代行其他角色权限。

主要文档：

- `docs/requirements.md`：业务目标、角色权限、状态流转、数据模型和完整接口规划；
- `docs/development-plan.md`：任务拆分、较新的实现进度、测试与部署目标；
- `docs/seaorm-conventions.md`：实体生成、SeaORM 使用与数据访问分层约定；
- `backend/README.md`、`frontend/README.md`：启动方式和模块约定。

注意：根 `README.md`、两个子项目 README 和需求文档中的部分“当前进度/待开发”标记已经滞后。判断某项功能是否已实现时，以当前源码、后端路由表和前端实际请求为准，不要把规划中的接口当成已有能力。

## 技术栈与运行架构

### 后端

- Rust 2024 edition，最低版本由 workspace 固定为 Rust 1.85；
- Cargo workspace，resolver 3；
- Axum 0.8 + Tokio，HTTP 服务默认监听 `127.0.0.1:8080`；
- SeaORM 2 + SQLx PostgreSQL 驱动；
- PostgreSQL，默认数据库名 `research_apply_db`；
- Argon2 密码哈希，JWT 会话，HttpOnly Cookie；
- `tower-http` 提供 CORS 和 HTTP trace，`tracing` 输出紧凑单行日志。

### 前端

- Next.js 16.3.6 App Router + React 19 + TypeScript strict mode；
- Tailwind CSS 4，经 `@tailwindcss/postcss` 接入；
- 登录表单使用 `react-hook-form`、`zod` 和 `@hookform/resolvers`；
- 图标使用 Iconify；
- 使用 npm 和已提交的 `package-lock.json`。

### 请求与认证链路

1. 浏览器中的交互页面主要是 Client Components。
2. 浏览器业务请求访问 Next.js 同源 `/api/*`；`frontend/next.config.ts` 用 rewrites 将其转发到 `API_PROXY_TARGET`，默认 `http://127.0.0.1:8080`。
3. 根登录页是动态 Server Component，直接通过 `NEXT_PUBLIC_API_BASE` 请求后端 `/health` 和 `/api/meta`。
4. 登录成功后，后端签发有效期 8 小时的 JWT，并写入名为 `research_apply_session` 的 HttpOnly、SameSite=Lax Cookie。后端也接受 `Authorization: Bearer <JWT>`。
5. `CurrentUser` 提取器在每次受保护请求中加载用户、启用状态和角色；业务归属和权限由后端服务再次检查。
6. 后端通过 SeaORM 连接 PostgreSQL，连接池最大 10、最小 1。

前端 `localStorage` / `sessionStorage` 只保存用于页面展示的角色和账号资料，不是认证凭证，也不能作为权限边界。`frontend/src/proxy.ts` 在没有 `research_apply_session` 时把业务页重定向到 `/login`（Next.js 16 用 proxy 代替 middleware）。这只检查 Cookie 是否存在，不校验 JWT，也还没有按角色拦截页面；直接访问业务路由不能被当作安全控制。

## 目录与模块划分

```text
research-apply/
├── docs/                         需求与开发计划
├── backend/                      Cargo workspace
│   ├── Cargo.toml                workspace 成员、Rust 版本和 lint
│   ├── server/                   research-apply-server
│   │   └── src/
│   │       ├── main.rs、lib.rs    启动与公共模块导出
│   │       ├── config.rs         环境变量、Cookie 名、会话时长
│   │       ├── state.rs          AppState 与数据库连接池
│   │       ├── error.rs          AppError、字段错误、HTTP 映射
│   │       ├── auth/             Argon2、JWT、CurrentUser、角色守卫
│   │       ├── routes/           Axum 路由表和 HTTP handler
│   │       ├── services/         认证、申报书、部门、用户和批次业务逻辑
│   │       ├── support/          审计、时间、日志、可空 PATCH 字段反序列化
│   │       ├── seed.rs           演示数据实现
│   │       └── bin/seed.rs       种子命令入口
│   ├── migration/                SeaORM 迁移 CLI
│   │   ├── src/                  迁移注册、SQL 执行与单元测试
│   │   └── sql/001_up.sql        初始 DDL、字典和 2026 批次
│   └── entity/                   sea-orm-cli 生成的 23 个实体，不要手改
└── frontend/                     Next.js 应用
    ├── next.config.ts            `/api/*` 同源代理
    ├── package.json              dev/build/start/lint 脚本
    └── src/
        ├── app/                  App Router 页面和全局样式
        ├── components/           登录、工作台、教师填报、公共 UI
        └── lib/                  API、类型、登录资料、栏目与校验
```

后端保持“route 解析 HTTP 输入并调用 service，service 执行业务规则和数据库写入”的分层。目前教师申报逻辑集中在较大的 `backend/server/src/services/applications.rs`；其保存策略是逐栏目写入，多行子表在事务内整组删除后重建，序号由服务层生成。

前端关键文件：

- `src/lib/api.ts`：通用 `apiSend`、`ApiError`、登录和健康/元信息请求；
- `src/lib/applications.ts`：教师申报 API 与 TypeScript 类型；
- `src/lib/admin.ts`：部门、用户、批次管理 API 与 TypeScript 类型；
- `src/components/admin-workspace.tsx`：真实部门、用户、批次管理表格与编辑弹窗；
- `src/components/teacher-workspace.tsx`：教师列表、工作台和状态展示；
- `src/components/teacher-apply.tsx`：社科/自科分步填报、预算、提交、撤回和删除草稿；
- `src/components/workspace-shell.tsx`：各角色工作台外壳与退出；
- `src/lib/login.ts`：角色导航、演示预设和展示资料存储；
- `src/lib/checks.ts`：基本情况的浏览器端校验；
- `src/app/globals.css`：Tailwind 主题变量和公共表单/表格样式。

路径别名 `@/*` 指向 `frontend/src/*`。

## 当前实现范围

已接入真实后端和数据库：

- `/health`、`/api/meta`；
- 登录、退出、当前用户；
- 经费科目查询；
- 教师创建申报书、本人列表和详情；
- 社科、自科、双高专项的逐栏目保存；
- 参加者、成果、进度、设备、预算和推荐人保存；
- 提交校验、确认承诺后提交、未系审前撤回、删除草稿；
- 部门列表、新增、修改、删除及关联数据保护；
- 用户按部门、角色、关键字筛选，新建、修改、删除、停用、重置密码和多角色分配；已关联申报或审核数据的用户禁止删除；
- 批次列表、新建、修改时间、开放/关闭和当前批次查询；首次提交校验申报书所属批次；
- 管理员工作台的部门、用户、批次页面均使用真实 API；
- 登录、教师申报、基础数据管理、系审核、科研处审核、专家指派和立项的审计写入；
- 系审核：本部门待审列表、本人已审列表、全文查看，以及通过、退回、不通过。管理员可审全院。重复审核返回 409；
- 科研处审核：全院待审、封面编号、退回、不通过、按研究专长指派专家、撤销未评审指派、盲审汇总、立项或不予立项。申报人可以看到结论说明；
- 专家盲审：待评和已评只含类别与名称；详情是独立匿名内容。评分 0–100，提交后不可修改。未指派的专家读取申报书返回 404；
- 审核时间线：`GET /api/applications/{id}/timeline`。教师、本部门系审核人、科研处和管理员可查看。申请人与系审核人看到的专家写成「专家 1 / 2 / 3」，响应不含 `reviewer_id`。科研处和管理员可以看到专家姓名；
- 个人资料：`GET/PATCH /api/profile`。本人可改性别、民族、出生年月、行政职务、专业技术职务、研究专长、学历、学位、手机和邮箱。新建申报书复制这份资料，已有申报书不变；
- 统计：`GET /api/stats/overview` 与 `GET /api/stats/export`。科研处和管理员按批次查看类别、部门、状态的申报数量和申请经费，并导出 Excel。页面在 `/stats`。
- 附件：`POST /api/applications/{id}/attachments` 上传，`GET/DELETE /api/applications/{id}/attachments/{attachment_id}` 下载和删除。仅 PDF/JPG/PNG，单个不超过 20 MB，存储键为 UUID。文件写在 `ATTACHMENT_DIR`（默认 `data/attachments`）。负责人在可编辑状态上传和删除；申报人、本部门系审核人、科研处和管理员可下载全部类型；已指派专家看不到签字件，未指派专家读取返回 404。申报详情和匿名详情都带 `attachments`，匿名详情不含签字件。
- 申报书 PDF：`GET /api/applications/{id}/form.pdf`。按 2026 年社会科学类、自然科学类纸质申报书排版，双高专项使用自然科学类版式。经费由万元换算为元。教师可导出本人申报；本部门系审核人、科研处和管理员可导出；专家返回 403。服务器需要中文字体，可用 `PDF_FONT` 指定。

仍属页面演示或尚未实现：

- 通知查询和拟立项公示名单尚未实现；
- 完整前端路由鉴权、申报书独立只读视图和浏览器端到端测试尚未完成。

不要把演示工作台中的行为当作后端业务事实，也不要在修改这些页面时误以为已有对应 API。

## 本地启动

环境要求以项目 README 为准：Rust 1.85+、Node.js 20+、PostgreSQL 14+。只有重新生成 SeaORM 实体时才需要 `sea-orm-cli`。

```bash
createdb research_apply_db

cd backend
cp .env.example .env
# 修改 DATABASE_URL，并设置自己的 JWT_SECRET
cargo run -p migration -- up
cargo run -p research-apply-server --bin seed
cargo run -p research-apply-server
```

另开终端：

```bash
cd frontend
cp .env.example .env.local
npm ci
npm run dev
```

打开 `http://localhost:3000`。根路径就是登录页，`/login` 和旧的 `/applications/{social,natural}/new` 会重定向到当前入口。

后端环境变量：

- `DATABASE_URL`：必填；
- `JWT_SECRET`：必填，生产环境必须换成私密随机值；
- `SERVER_BIND`：默认 `127.0.0.1:8080`；
- `CORS_ORIGIN`：默认 `http://localhost:3000`；
- `COOKIE_SECURE`：HTTPS 下设为 `true` 或 `1`；
- `RUST_LOG`：默认 `info,tower_http=info`。

前端环境变量：

- `NEXT_PUBLIC_API_BASE`：Server Components 直连后端，默认 `http://127.0.0.1:8080`；
- `API_PROXY_TARGET`：Next.js `/api/*` 转发目标，默认相同地址。

健康检查：

```bash
curl http://127.0.0.1:8080/health
curl http://127.0.0.1:8080/api/meta
```

`seed` 可重复执行，但会重置测试账号密码、启用这些账号并重新开放 2026 批次；不要在含真实数据的环境随意执行。测试账号和密码由命令输出，也写在 `server/src/seed.rs` 和 `docs/科研处测试说明.md`，登录页不预填。这些账号绝不能作为生产凭据。

## 构建、检查与数据库命令

在 `backend/` 执行：

```bash
cargo check --workspace
cargo fmt --all
cargo fmt --all -- --check
cargo clippy --workspace --all-targets
cargo test --workspace
```

迁移命令：

```bash
cargo run -p migration -- status
cargo run -p migration -- up
cargo run -p migration -- down
cargo run -p migration -- fresh   # 危险：删表重建并清空数据
cargo run -p migration -- generate add_review_score
```

新增迁移后，在 `migration/src/lib.rs` 注册。初始迁移已经冻结，结构调整必须新增迁移，不要继续修改 `m20260929_000001_init_schema` 或 `001_up.sql` / `001_down.sql`。

迁移后重新生成实体：

```bash
sea-orm-cli generate entity \
  -u "$DATABASE_URL" \
  -o entity/src \
  --lib \
  --with-serde both
```

在 `frontend/` 执行：

```bash
npm run dev
npm run lint
npm run build
npm run start
```

仓库没有 Prettier 配置、独立前端类型检查脚本或测试脚本；`npm run build` 会执行 Next.js 构建和 TypeScript 检查。

## 数据与业务约定

数据库当前有 23 张表、10 个 PostgreSQL 枚举：

- 账号：`departments`、`users`、`user_roles`；
- 批次：`application_cycles`；
- 公共申报：`applications`、`application_members`、`recommenders`、`application_attachments`、`budget_subjects`、`application_budgets`；
- 社会科学：`ss_*` 四表；
- 自然科学：`ns_*` 五表；
- 审核：`expert_assignments`、`reviews`；
- 通知与审计：`notifications`、`audit_logs`。

迁移 SQL 以单独一行的 `-- @@` 作为语句分隔标记，新增 SQL 时必须保留该格式。

核心约束：

- 不使用触发器或存储函数；数据库只负责主键、外键、唯一、`CHECK`、`NOT NULL` 等声明式约束，业务规则在 Rust 服务层实现；
- `updated_at` 没有数据库自动更新机制，写服务必须显式维护；不要把重新生成实体后会丢失的手写逻辑放进 `entity/`；
- 多表写入放在同一事务中；状态更新带旧状态条件，影响行数为 0 时返回 409；
- 只有 `draft`、`dept_returned`、`office_returned` 可编辑；
- 社科栏目只用于 `social`，自科栏目用于 `natural` 和 `dual_high`；
- 金额使用十进制数，API 以字符串传输，单位按栏目区分（预算和指标为万元，设备单价为元），不要改成浮点金额；
- 参加者最多 5 人，阶段成果最多 10 项、最终成果最多 2 项，设备单价不得低于 5000 元；
- 预算保存只接受当前类别的 `fillable` 科目；服务端响应计算直接费用和总合计，间接费用父行按字典配置填写，绩效支出不重复计入合计；
- `reviews` 和 `audit_logs` 按只追加记录设计，不修改历史审核和审计。

需求中的完整状态机和校验清单见 `docs/requirements.md` 第四、六节；修改规则时同步更新该文档。表结构变化还要新增迁移、重新生成实体并更新需求第六节。

## 代码风格与接口约定

### Rust

- 使用 rustfmt 默认风格；workspace 禁止 `unsafe_code`，Clippy 对 `unwrap_used` 发出警告；
- 错误统一使用 `AppError`，不要向客户端泄露数据库或内部错误细节；
- 错误响应为 `{ "error": "中文说明", "fields": [...] }`，字段校验使用 422，唯一约束冲突使用 409；
- handler 保持轻量，权限、归属、状态流转和事务放在 service；
- 日志和注释主要使用中文；不要记录密码、JWT 或敏感申报内容。

### TypeScript / React

- 使用双引号、TypeScript 严格类型和现有函数组件风格；
- 默认使用 Server Components，只有交互组件添加 `"use client"`；
- 浏览器请求统一走 `apiSend`，新增类型放在对应 `lib` 模块并与后端 DTO 同步；
- 登录表单沿用 react-hook-form + zod；教师填报目前是受控组件和 `useState`，基本情况校验在 `lib/checks.ts`，不要假设所有栏目已有 zod schema；
- 样式沿用 Tailwind 类和 `globals.css` 中的主题变量/公共类；
- 前后端校验都以 `docs/requirements.md` 6.3 为业务来源，但最终必须以后端校验为准。

## 测试策略与当前基线

当前已有 33 个 Rust 单元测试，均不需要数据库：

- migration 3 个：初始 SQL 覆盖核心表、无触发器/函数、down SQL 覆盖清理；
- server 30 个：密码哈希、JWT、角色守卫、申报校验、部门/用户/批次输入校验、状态机合法流转、系审核结论、科研处编号与回避、专家评分和匿名序列化。

另有 `server/tests/admin_api.rs`、`server/tests/department_reviews_api.rs`、`server/tests/review_flow_api.rs` 和 `server/tests/teacher_flow_api.rs` 四个数据库路由测试，使用 `tower::ServiceExt::oneshot`。前三个使用演示库，显式运行时会清理自建数据并恢复临时调整的批次状态。`teacher_flow_api` 使用独立测试库：从 `DATABASE_URL` 推导出以 `_test` 结尾的库名（也可用 `TEST_DATABASE_URL` 指定），自动建库、执行迁移，并在每个用例前清掉业务数据。它覆盖登录、401、403、两类申报书提交、越权、个人资料带入新申报书，以及统计合计和 Excel 导出。

```bash
cd backend
cargo test --workspace
cargo test -p research-apply-server --test admin_api -- --ignored --test-threads=1
cargo test -p research-apply-server --test department_reviews_api -- --ignored --test-threads=1
cargo test -p research-apply-server --test review_flow_api -- --ignored --test-threads=1
cargo test -p research-apply-server --test teacher_flow_api -- --ignored --test-threads=1
```

目前尚无前端单元测试或 Playwright 测试，也没有 CI 配置，不要把这些后续目标当作现有设施。

2026-10-26 对当前源码实际执行的结果：

- `cargo test --workspace`：通过，20 个测试，数据库路由测试默认忽略；
- 管理数据库路由测试：显式运行通过；
- `cargo clippy --workspace --all-targets`：通过；
- `npm run build`：通过；
- `cargo fmt --all -- --check`：通过；
- `npm run lint`：未通过，`teacher-apply.tsx`、`teacher-workspace.tsx`、`workspace-shell.tsx` 各有一个既有 `react-hooks/set-state-in-effect` 错误；本次管理页和 `lib/admin.ts` 定向 ESLint 通过。

2026-10-27 科研处、盲审和立项接入后：

- `cargo test --workspace`：通过，33 个单元测试，三个数据库路由测试默认忽略；
- `cargo test -p research-apply-server --test review_flow_api -- --ignored --test-threads=1`：通过；
- `cargo clippy -p research-apply-server -p migration --all-targets --offline -- -D warnings`：通过；
- `frontend/node_modules/.bin/tsc --noEmit`：通过；
- 科研处和专家页面的定向 ESLint 通过。`teacher-workspace.tsx` 的既有 `react-hooks/set-state-in-effect` 仍在。

2026-11-10 个人资料、统计和独立测试库接入后：

- `cargo test -p research-apply-server --test teacher_flow_api --offline -- --ignored --test-threads=1`：通过，4 个测试；
- `frontend/node_modules/.bin/tsc --noEmit`：通过；
- 个人资料页和统计页的定向 ESLint 通过。

修改代码后仍应运行上述检查，并区分既有基线问题与本次新增问题。

## 安全注意事项

- `.env`、`.env.local` 和依赖/构建目录均被忽略；不要提交真实数据库 URL、JWT 密钥或其他凭据；
- 生产必须更换 `JWT_SECRET` 和全部演示密码，启用 HTTPS 与 `COOKIE_SECURE=true`；
- 所有新接口都必须在后端校验角色、数据归属、账号启用状态和当前申报状态，不能依赖前端菜单或角色选择；
- 专家接口应使用独立匿名 DTO，结构上排除负责人、部门、联系方式、参加者、推荐人和签名附件；申请人响应不得暴露专家 `reviewer_id`；
- 内部错误只记录到服务端日志，客户端返回通用中文错误；
- 附件限定 PDF/JPG/PNG、单文件不超过 20 MB、UUID 存储键和下载鉴权；专家响应不得包含签字件；
- `migration fresh`、`down` 和演示种子都有数据破坏性，执行前确认目标数据库。

## 部署现状

仓库目前没有 Dockerfile、Compose 文件、反向代理配置、备份脚本或 CI/CD 工作流。生产前端只能按现有脚本使用 `npm run build` 后 `npm run start`，Next.js 尚未配置 standalone 输出。

`docs/requirements.md` 和 `docs/development-plan.md` 规划的 PostgreSQL + backend + frontend + nginx Compose 部署、HTTPS、日志落盘、每日备份并保留 30 天均为后续任务，不是现有部署能力。实现部署变更时，应同时更新启动文档和环境变量示例。

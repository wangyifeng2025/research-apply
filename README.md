# 院级科研项目申报系统

黔西南民族职业技术学院科研处的院级科研项目在线申报与审核系统。教师在线填写社会科学类或自然科学类申报书，经系审核、科研处审核、专家盲审后立项。

- 需求规格说明：[docs/requirements.md](docs/requirements.md)
- 开发计划：[docs/development-plan.md](docs/development-plan.md)
- 后端说明：[backend/README.md](backend/README.md)
- 前端说明：[frontend/README.md](frontend/README.md)

## 功能概览

| 角色 | 主要功能 |
|---|---|
| 教师 | 新建申报书（B.社会科学类 / A.自然科学类 / D."双高"校建设专项），分栏目暂存，确认承诺后提交，退回后修改重报 |
| 系审核人 | 审核本部门申报书，逐项确认"所在部门意见"四项内容，给出通过 / 退回 / 不通过 |
| 科研处 | 二级审核并填写封面编号，指派盲审专家，汇总盲审结果决定立项，统计 |
| 专家 | 匿名查看被指派的申报书并提交评审意见 |
| 管理员 | 部门、用户、角色、申报批次维护 |

申报状态流转：

```text
草稿 → 已提交 → 系审核通过 → 科研处审核通过 → 专家盲审中 → 立项 / 不予立项
          ↘ 退回（可重报） / 不通过   ↘ 退回（可重报） / 不通过
```

## 技术栈

| 层 | 选型 |
|---|---|
| 后端 | Rust（edition 2024）、Axum 0.8、SeaORM 2.0、Tokio、tower-http |
| 数据库 | PostgreSQL，库名 `research_apply_db` |
| 前端 | Next.js 16（App Router）、React 19、TypeScript、Tailwind CSS 4 |

## 目录结构

```text
research-apply/
├── docs/
│   └── requirements.md        需求规格说明
├── backend/                   Cargo workspace
│   ├── server/                Axum HTTP 服务（research-apply-server）
│   ├── migration/             SeaORM 迁移，SQL 在 migration/sql/
│   └── entity/                sea-orm-cli 生成的实体
└── frontend/                  Next.js 前端
    └── src/
        ├── app/               页面路由
        ├── components/        公共组件
        └── lib/               接口调用与申报书栏目定义
```

## 快速开始

### 1. 环境要求

- Rust 1.85 及以上
- Node.js 20 及以上
- PostgreSQL 14 及以上（迁移用到 `gen_random_uuid()`，PostgreSQL 13 起内置）
- `sea-orm-cli`（仅重新生成实体时需要）：`cargo install sea-orm-cli`

### 2. 准备数据库

```bash
createdb research_apply_db
```

### 3. 启动后端

```bash
cd backend
cp .env.example .env            # 按本机修改 DATABASE_URL
cargo run -p migration -- up    # 建表、写入经费科目和 2026 批次
cargo run -p research-apply-server --bin seed
cargo run -p research-apply-server
```

服务默认监听 `http://127.0.0.1:8080`，检查：

```bash
curl http://127.0.0.1:8080/health
curl http://127.0.0.1:8080/api/meta
```

### 4. 启动前端

```bash
cd frontend
cp .env.example .env.local      # NEXT_PUBLIC_API_BASE 指向后端地址
npm install
npm run dev
```

打开 <http://localhost:3000> 即进入登录页。前端把 `/api` 同源转发到后端，登录 Cookie 因此可以写入浏览器。教师进入「我的申报」，系审核人、科研处、专家分别进入对应审核台，管理员进入 `/admin`。

## 当前进度

| 模块 | 状态 |
|---|---|
| 数据库迁移（23 张表、声明式约束、不含触发器，经费科目字典、2026 批次） | 已完成 |
| SeaORM 实体 | 已生成 |
| 后端骨架：配置、错误处理、CORS、日志、`/health`、`/api/meta` | 已完成 |
| 前端骨架：首页、申报入口、两类申报书栏目说明、三个审核台静态页 | 已完成 |
| 登录与权限 | 已完成：种子账号、JWT Cookie、按角色进入页面 |
| 申报书填写与提交接口、表单 | 待开发 |
| 三级审核与盲审接口、页面 | 待开发 |
| 统计、导出、通知 | 待开发 |

完整开发计划见 [docs/development-plan.md](docs/development-plan.md)，待确认事项见 [docs/requirements.md](docs/requirements.md) 第十一节。

## 常用命令

| 目的 | 命令（在对应目录执行） |
|---|---|
| 后端检查 | `cargo check --workspace` |
| 后端测试 | `cargo test --workspace` |
| 后端 lint | `cargo clippy --workspace --all-targets` |
| 演示数据 | `cargo run -p research-apply-server --bin seed` |
| 迁移状态 | `cargo run -p migration -- status` |
| 重建数据库 | `cargo run -p migration -- fresh` |
| 前端 lint | `npm run lint` |
| 前端构建 | `npm run build` |

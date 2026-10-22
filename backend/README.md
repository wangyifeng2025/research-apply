# 科研申报系统 · 后端

Rust + Axum + SeaORM + PostgreSQL。业务需求见 [../docs/requirements.md](../docs/requirements.md)。

## Workspace 结构

| crate | 路径 | 说明 |
|---|---|---|
| `research-apply-server` | `server/` | Axum HTTP 服务 |
| `migration` | `migration/` | SeaORM 迁移 CLI；实际 DDL 写在 `migration/sql/001_up.sql` 和 `001_down.sql`，由 `m20260929_000001_init_schema.rs` 按 `-- @@` 分隔逐条执行 |
| `entity` | `entity/` | `sea-orm-cli generate entity` 生成的实体，不要手改 |

```text
server/src/
├── main.rs        启动：读取 .env、初始化日志、连接数据库、监听端口
├── bin/seed.rs    演示账号与开放 2026 批次
├── config.rs      环境变量
├── state.rs       AppState（数据库连接池）
├── error.rs       AppError 及统一错误响应
├── auth/          密码、JWT Cookie、CurrentUser、require_role
├── services/      登录业务
├── support/       审计写入、时间
└── routes/
    ├── mod.rs     路由表、CORS、TraceLayer
    ├── health.rs  GET /health
    ├── meta.rs    GET /api/meta
    └── auth.rs    登录、退出、当前用户
```

## 配置

复制 `.env.example` 为 `.env`：

| 变量 | 必填 | 默认值 | 说明 |
|---|---|---|---|
| `DATABASE_URL` | 是 | — | 如 `postgres://user:password@127.0.0.1:5432/research_apply_db` |
| `JWT_SECRET` | 是 | — | 签发登录 Cookie 的密钥，生产环境必须更换 |
| `SERVER_BIND` | 否 | `127.0.0.1:8080` | 监听地址 |
| `CORS_ORIGIN` | 否 | `http://localhost:3000` | 允许跨域的前端地址，登录需允许携带 Cookie |
| `COOKIE_SECURE` | 否 | 关闭 | 设为 `true` 时登录 Cookie 只在 HTTPS 下发送 |
| `RUST_LOG` | 否 | `info,tower_http=info` | 日志级别 |

`server` 和 `migration` 都会读取当前目录下的 `.env`，以下命令均在 `backend/` 目录执行。

## 数据库迁移

```bash
cargo run -p migration -- up        # 执行未应用的迁移
cargo run -p migration -- status    # 查看状态
cargo run -p migration -- down      # 回滚最近一次
cargo run -p migration -- fresh     # 删除全部表后重建（会清空数据）
```

初始迁移会创建：

- 10 个枚举类型：`user_role`、`project_class`、`cycle_status`、`application_status`、`review_stage`、`review_decision`、`output_kind`、`budget_family`、`attachment_kind`、`notification_kind`；
- 23 张表：账号（`departments`、`users`、`user_roles`）、批次（`application_cycles`）、申报公共（`applications`、`application_members`、`recommenders`、`application_attachments`、`budget_subjects`、`application_budgets`）、社会科学（`ss_*` 4 张）、自然科学（`ns_*` 5 张）、审核（`expert_assignments`、`reviews`）、通知与审计（`notifications`、`audit_logs`）；
- 声明式约束：主键、外键、唯一、`CHECK`，例如参加者序号 1–5 且唯一（即最多 5 人）、非草稿必须填齐封面必填项、专家评分 0–100、退回必须有意见；
- 种子数据：两类申报书的全部经费科目、"2026年院级科研项目申报"批次（状态 `draft`）。

**不使用触发器和存储函数**，`updated_at` 也不会自动刷新。业务规则、权限和状态流转都在服务层实现，规则清单见需求文档 6.3，数据库约束清单见 6.2。

### 新增迁移

```bash
cargo run -p migration -- generate add_review_score
```

在生成的文件中实现 `up` / `down`，并在 `migration/src/lib.rs` 的 `migrations()` 中注册。已经上线的迁移文件不要修改，改动一律新增迁移。

### 重新生成实体

迁移执行完后：

```bash
sea-orm-cli generate entity \
  -u "$DATABASE_URL" \
  -o entity/src \
  --lib \
  --with-serde both
```

## 运行

先写入演示数据（可重复执行，会重置演示账号密码并重新开放 2026 批次）：

```bash
cargo run -p research-apply-server --bin seed
cargo run -p research-apply-server
```

| 接口 | 说明 |
|---|---|
| `GET /health` | 健康检查；数据库不可用时返回 500，响应体不包含 SQL |
| `GET /api/meta` | 当前数据库名、审核流程、项目类别、最新批次、经费科目数，供前端首页使用 |
| `POST /api/auth/login` | 校验账号密码与 `is_active`，写入 HttpOnly Cookie，有效期 8 小时 |
| `POST /api/auth/logout` | 清除登录 Cookie |
| `GET /api/auth/me` | 当前用户资料与角色；无令牌返回 401 |

演示账号见种子命令的输出。密码使用 argon2，日志不记录密码。后续需要按角色拦截的接口调用 `require_role`：管理员可执行任一角色的操作，角色不符返回 403。

规划中的接口见需求文档第七节。

## 开发约定

- 错误统一走 `AppError`，响应体为 `{ "error": "中文说明", "fields": [...] }`，不向前端暴露数据库错误细节；唯一约束冲突返回 409；
- 状态流转只在服务层做，更新时带旧状态条件，多表写入放进事务；
- 专家相关接口使用独立 DTO，不包含负责人、参加者、推荐人等身份字段；
- 迁移中不写触发器和存储函数；业务校验在服务层完成并返回字段级错误（422），数据库约束只作兜底，被触发说明服务层有遗漏；
- 更新数据时写 `updated_at`，建议在各实体的 `ActiveModelBehavior::before_save` 中统一处理（重新生成实体会覆盖这些文件，届时需要重新加回）；
- workspace 禁止 `unsafe`，`unwrap()` 会触发 clippy 警告。

## 检查与测试

```bash
cargo fmt --all
cargo clippy --workspace --all-targets
cargo test --workspace
```

`migration` 中已有单元测试，检查 SQL 脚本覆盖两类申报书的核心表。

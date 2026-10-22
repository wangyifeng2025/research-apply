# 科研申报系统 · 前端

Next.js 16（App Router）+ React 19 + TypeScript + Tailwind CSS 4。业务需求见 [../docs/requirements.md](../docs/requirements.md)。

## 启动

```bash
cp .env.example .env.local
npm install
npm run dev
```

打开 <http://localhost:3000> 即进入登录页，左侧显示当前批次和数据库连接。后端需先启动（见 [../backend/README.md](../backend/README.md)）。

| 变量 | 默认值 | 说明 |
|---|---|---|
| `NEXT_PUBLIC_API_BASE` | `http://127.0.0.1:8080` | 服务端组件读取 `/health`、`/api/meta` 的后端地址 |
| `API_PROXY_TARGET` | `http://127.0.0.1:8080` | 浏览器请求 `/api/*` 时的转发目标，登录走这条路径以携带 Cookie |

## 目录结构

```text
src/
├── app/
│   ├── layout.tsx                     根布局
│   ├── page.tsx                       网站入口：登录页
│   ├── login/page.tsx                 转到 /
│   ├── globals.css                    主题色变量与全局样式
│   ├── applications/
│   │   ├── page.tsx                   我的申报
│   │   ├── social/new/page.tsx        新建社会科学类申报
│   │   └── natural/new/page.tsx       新建自然科学类申报
│   └── reviews/
│       ├── department/page.tsx        系审核
│       ├── office/page.tsx            科研处审核
│       └── expert/page.tsx            专家盲审
├── components/
│   ├── cover-fields.tsx               申报书封面字段
│   ├── section-list.tsx               申报书栏目列表
│   └── review-desk.tsx                三个审核台的公共页面
└── lib/
    ├── api.ts                         后端接口与类型
    └── sections.ts                    两类申报书的栏目定义及系审核四项确认
```

`lib/sections.ts` 中每个栏目都标注了对应的数据库表，后续做表单时以它为栏目清单。

## 当前进度

- 已完成：登录页（接入 `POST /api/auth/login`，左侧显示批次和数据库连接）、按角色进入的工作台；
- 待开发：分栏目表单、子表增删行组件、预算表、审核操作、专家匿名视图、统计看板、管理页面。

规划的路由见需求文档第八节。

## 开发约定

- 默认使用服务端组件，只有需要交互的表单和组件加 `"use client"`；
- 接口调用集中在 `lib/api.ts`，类型与后端响应保持一致；
- 表单校验规则与后端一致：项目名称 ≤ 40 字，关键词 ≤ 5 个，参加者 ≤ 5 人，阶段成果 ≤ 10 项、最终成果 ≤ 2 项，设备单价 ≥ 5000 元；
- 专家页面只使用匿名接口返回的数据，不在前端拼接申请人信息；
- 颜色使用 `globals.css` 中的 CSS 变量（如 `--color-ink`、`--color-cinnabar`）。

## 命令

| 命令 | 说明 |
|---|---|
| `npm run dev` | 开发服务器 |
| `npm run build` | 生产构建 |
| `npm run start` | 运行生产构建 |
| `npm run lint` | ESLint |

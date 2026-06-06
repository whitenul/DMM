# Desk Manager

Windows 桌面应用启动器与管理工具，基于 Tauri 2.0 + Vue 3 + TypeScript 构建。

## 功能特性

- **应用启动器**：通过可自定义的分类网格组织并启动应用程序
- **主题系统**：完整的主题支持，包括明暗模式、自定义强调色、主题导入导出
- **透明度控制**：可调节窗口透明度（5%–95%），由 CSS 变量驱动实现平滑过渡
- **背景图片**：支持自定义背景图片及模糊效果
- **全局搜索**：通过全局快捷键快速搜索并启动应用
- **拖拽排序**：分类和项目支持拖拽重新排序
- **自动扫描**：自动扫描开始菜单或自定义文件夹中的应用并导入
- **UWP 应用支持**：扫描并导入 Microsoft Store 安装的 UWP 应用
- **多语言**：支持简体中文和英文

## 技术栈

| 层级 | 技术 | 说明 |
|------|------|------|
| 桌面框架 | Tauri 2.0 | Rust 后端 + WebView2 前端 |
| 前端框架 | Vue 3 + TypeScript | Composition API + Pinia 状态管理 |
| UI 风格 | Fluent Design | 毛玻璃效果、圆角、Win11 风格 |
| 数据库 | SQLite (rusqlite) | 应用数据、分类、快捷方式 |
| 配置文件 | TOML | 用户偏好、窗口位置、主题配置 |
| 构建工具 | Vite | 前端构建与热更新 |
| 目标平台 | Windows 10+ | WebView2 已预装 |

## 安装与运行

### 环境要求

- [Node.js](https://nodejs.org/) v20+
- [pnpm](https://pnpm.io/) v10+
- [Rust](https://www.rust-lang.org/) latest stable
- Windows 10 或更高版本

### 开发模式

```bash
# 安装前端依赖
pnpm install

# 启动开发服务器（含 Rust 后端热编译）
pnpm tauri dev
```

### 生产构建

```bash
# 构建安装包
pnpm tauri build
```

构建产物位于 `src-tauri/target/release/bundle/`。

## 项目结构

```
Desk Manager/
├── src/                          # 前端源码
│   ├── assets/styles/            # 全局 CSS 与设计 Token
│   │   ├── tokens.css            # 基础设计 Token（间距、圆角、阴影等）
│   │   └── variables.css         # 语义变量与主题系统
│   ├── components/               # Vue 组件
│   │   ├── category/             # 分类相关组件
│   │   ├── common/               # 通用组件（对话框、图标、右键菜单等）
│   │   ├── item/                 # 项目相关组件
│   │   ├── layout/               # 布局组件（标题栏、侧边栏、内容区）
│   │   ├── search/               # 搜索组件
│   │   └── settings/             # 设置相关组件
│   ├── composables/              # Vue Composables
│   │   ├── useI18n.ts            # 国际化
│   │   ├── useSortable.ts        # 拖拽排序
│   │   ├── useTauriCommand.ts    # Tauri 命令封装
│   │   └── useWindowClose.ts     # 窗口关闭行为
│   ├── pages/                    # 页面组件
│   │   ├── main/                 # 主页面
│   │   └── settings/             # 设置页面
│   ├── router/                   # Vue Router 配置
│   ├── stores/                   # Pinia 状态管理
│   │   ├── category.ts           # 分类状态
│   │   ├── item.ts               # 项目状态
│   │   ├── scan.ts               # 扫描状态
│   │   ├── search.ts             # 搜索状态
│   │   ├── settings.ts           # 设置状态
│   │   ├── theme.ts              # 主题状态
│   │   ├── toast.ts              # 提示消息状态
│   │   └── ui.ts                 # UI 状态
│   ├── themes/                   # 预装主题定义
│   ├── types/                    # TypeScript 类型定义
│   └── utils/                    # 工具函数
├── src-tauri/                    # Rust 后端
│   ├── src/                      # 主入口
│   │   ├── lib.rs                # 应用初始化与 Tauri Command
│   │   ├── main.rs               # 程序入口
│   │   └── logging.rs            # 日志配置
│   ├── crates/                   # 工作区 Crate
│   │   ├── desk-core/            # 核心模块（数据库、配置、领域模型）
│   │   ├── desk-category/        # 分类管理
│   │   ├── desk-item/            # 项目管理
│   │   ├── desk-scan/            # 应用扫描
│   │   ├── desk-search/          # 搜索引擎
│   │   ├── desk-icon/            # 图标提取
│   │   ├── desk-settings/        # 设置管理
│   │   └── desk-web/             # Web 元数据抓取
│   └── capabilities/             # Tauri 权限配置
└── ...
```

## 主题系统

Desk Manager 采用三层 CSS 变量架构：

1. **Token 层**（`tokens.css`）：基础设计值（间距、圆角、阴影）
2. **主题层**（`--theme-*`）：运行时由 JS 注入的颜色值
3. **语义层**（`--color-*`）：组件引用的语义化变量

### 透明度系统

窗口透明度由 `--app-opacity` CSS 变量驱动（范围 5%–95%），通过 `color-mix()` 和 `calc()` 实现平滑过渡：

- **bg-layer**：`opacity: calc(1 - --app-opacity)` — 背景图片/底色随透明度淡出
- **glass-layer**：`color-mix()` 公式 — UI 层不透明度同步降低，5% 下限保底
- **backdrop-filter**：`blur(calc(base × (1 - opacity) × (1 - has-bg-image)))` — 模糊随透明度渐变

### 自定义主题

支持导入/导出 JSON 格式的主题文件，通过 Zod 校验确保结构正确。

## 配置

应用配置存储在 `%APPDATA%/Desk Manager/config.toml`，主要配置项：

| 配置项 | 说明 | 默认值 |
|--------|------|--------|
| `appearance.theme` | 明暗模式 (light/dark/system) | system |
| `appearance.theme_id` | 主题 ID | default |
| `appearance.effect` | 窗口效果 (mica/acrylic/none/auto) | auto |
| `appearance.app_opacity` | 窗口透明度 (0.05–0.95) | 0 |
| `appearance.accent_source` | 强调色来源 (system/theme/custom) | system |
| `appearance.language` | 界面语言 (zh-CN/en) | zh-CN |
| `window.width/height` | 窗口尺寸 | 900/620 |
| `close_behavior` | 关闭行为 (ask/minimize_to_tray/quit) | ask |

## 开发指南

### 代码规范

- **前端注释**：JSDoc 格式，中文描述，标准详细度（公共 API 和非显而易见逻辑）
- **Vue 模板**：区域标签使用 HTML 注释（如 `<!-- 主题设置 -->`）
- **分区标记**：使用 `// --- 区域名 ---` 格式
- **Rust 代码**：遵循 `cargo clippy` 规范

### 常用命令

```bash
# 前端类型检查
npx vue-tsc --noEmit

# Rust 代码检查
cargo clippy --no-deps

# Rust 单元测试
cargo test

# 前端开发服务器
pnpm tauri dev
```

### 添加新的 Tauri Command

1. 在对应的 `src-tauri/crates/desk-*/src/lib.rs` 中实现命令函数，添加 `#[tauri::command]`
2. 在 `src-tauri/capabilities/default.json` 中添加对应权限
3. 前端通过 `useTauriCommand` composable 调用

## 贡献

1. Fork 本仓库
2. 创建功能分支：`git checkout -b feature/your-feature`
3. 提交更改：`git commit -m "Add your feature"`
4. 推送分支：`git push origin feature/your-feature`
5. 创建 Pull Request

### 提交规范

- `feat:` 新功能
- `fix:` 修复 Bug
- `refactor:` 重构
- `docs:` 文档
- `style:` 格式调整
- `chore:` 构建/工具变更

## License

ISC

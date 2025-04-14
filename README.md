# DeepSeek Chat Clone

本项目是一个模仿DeepSeek官网的聊天Web应用，完全由Cline+DeepSeek AI辅助开发完成。

## 项目特点

- 🚀 100% AI辅助开发 - 所有代码由Cline+DeepSeek协作完成
- 💬 模仿DeepSeek官网的聊天界面和交互
- ⚡ 现代化技术栈，前后端分离架构

## 技术栈

### 前端
- Next.js 14 (App Router)
- TypeScript
- Tailwind CSS
- React Hooks

### 后端
- Rust
- Axum Web框架
- 支持CORS跨域访问

## 本地运行

### 前端开发服务器

```bash
# 安装依赖
pnpm install

# 启动开发服务器
pnpm dev
```

前端将在 [http://localhost:3000](http://localhost:3000) 运行

### 后端服务器

```bash
# 进入后端目录
cd rest

# 启动Rust后端
cargo run
```

后端将在 [http://localhost:8000](http://localhost:8000) 运行

## 项目结构

```
.
├── app/                # Next.js前端页面
├── components/         # React组件
├── config/             # 应用配置
├── public/             # 静态资源
└── rest/               # Rust后端服务
    ├── src/
    │   └── main.rs     # 后端入口文件
    └── Cargo.toml      # Rust依赖配置
```

## 功能特性

- 对话历史记录
- 按时间分组显示
- 暗黑/明亮模式切换
- 响应式设计

## 开发说明

本项目展示了AI辅助开发的强大能力，所有代码均由Cline+DeepSeek协作完成，包括：
- 前端界面开发
- 后端API实现
- 跨域配置
- 交互效果优化

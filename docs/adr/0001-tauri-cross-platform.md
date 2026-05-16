# ADR 0001: 选用 Tauri 作为跨平台框架

## 状态

已接受

## 背景

眸润需同时支持 macOS 与 Windows，需评估跨平台桌面应用方案。

候选方案：
- Electron：生态成熟，但打包体积大（>100MB），内存占用高。
- Tauri：Rust 后端 + Web 前端，打包体积小（<10MB），启动快。
- Flutter Desktop：UI 一致，但桌面端生态尚弱，插件支持不足。
- .NET MAUI：Windows 支持好，macOS 体验逊，且绑定微软生态。

## 决策

选用 Tauri。

## 后果

- 正面：体积小、启动快、内存占用低，适合轻量提醒类工具。
- 正面：Web 前端技术栈（React + TypeScript）团队熟悉度高。
- 负面：Rust 学习曲线若团队无经验需适应。
- 负面：桌面端部分系统 API（如全局输入拦截）需自行实现或寻找插件。

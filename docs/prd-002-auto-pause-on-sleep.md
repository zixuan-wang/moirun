# PRD-002：系统自动暂停 —— 睡眠/锁屏时不提醒

## Problem Statement

眸润当前每秒轮询计时器触发喝水与护眼提醒。当用户合上笔记本盖子、系统进入睡眠/休眠或锁屏时，计时器仍在后台运行。待用户唤醒电脑后，已到期的提醒会立即弹出——此时用户尚未开始工作，弹窗不仅无意义，还可能打断用户唤醒后的首件事务。

用户期望：电脑睡眠/锁屏期间，眸润自动停止计时，唤醒/解锁后从断点续计，而非立即弹窗。

## Solution

眸润在后台监听系统级电源与锁屏事件。当检测到系统进入睡眠、休眠或锁屏状态时，自动冻结喝水与护眼两个计时器；当系统唤醒或解锁后，计算休眠时长并将下次提醒时间点向后推移同等时长，实现"眠则止、醒则续"的无感体验。

此功能恒开，无开关，不干扰现有手动勿扰模式。

## User Stories

1. 作为 macOS 用户，我合上笔记本盖子让系统睡眠后，眸润不应在唤醒后立即弹出护眼提醒，以免打断我唤醒后的工作流。
2. 作为 Windows 用户，我让电脑进入休眠模式后，眸润不应在恢复后立刻弹出喝水通知，因为睡眠期间我并未用眼。
3. 作为用户，我临时离开电脑让系统自动锁屏后，眸润应暂停计时，因为锁屏期间我不可能喝水或休息眼睛。
4. 作为用户，我唤醒电脑后，眸润的下次提醒时间应从断点继续，而非从零开始重新计时一个完整周期。
5. 作为用户，我在手动勿扰模式期间让电脑睡眠，唤醒后勿扰模式仍然有效，不受自动睡眠暂停的影响。
6. 作为用户，我不希望在托盘图标或菜单中看到睡眠暂停的状态指示，保持界面简洁。
7. 作为外接显示器用户，我合上笔记本盖子但外接屏仍工作时，眸润应继续正常计时，因为此时我仍在使用电脑。
8. 作为用户，我连续触发睡眠和锁屏（或反之）时，眸润不应重复补偿时长，只计算一次实际休眠时间。

## Implementation Decisions

### 模块划分

- **`power_monitor`（新建深模块）**：封装平台特定的电源与锁屏事件监听。对外暴露极简接口：注册 `on_suspend` 和 `on_resume` 回调。内部按平台条件编译实现——macOS 用 `NSWorkspace` 睡眠通知与分布式通知（`com.apple.screenIsLocked`）检测锁屏；Windows 用 `WM_POWERBROADCAST` 与 `WTSRegisterSessionNotification`。`timer.rs` 与 `lib.rs` 不感知平台细节。
- **`timer.rs`（修改）**：在 `TimerState` 中新增系统暂停状态字段（`system_pause_start: Option<Instant>` 与 `is_system_paused: bool`）。轮询循环在 `is_system_paused` 为真时跳过所有提醒触发。提供 `enter_system_pause()` 与 `exit_system_pause()` 方法，后者计算休眠时长并向后推移 `water_next` 与 `eye_next`。
- **`lib.rs`（修改）**：在 `setup` 中初始化 `power_monitor`，将 `AppHandle` 与 `Arc<Mutex<TimerState>>` 传入，注册 suspend/resume 回调以操作计时器状态。

### 自动暂停与手动勿扰模式的关系

二者完全解耦。`is_system_paused` 与 `DoNotDisturbState` 独立存在：
- 系统暂停仅冻结计时器、推迟下次提醒时间点，不改变 DND 状态。
- 若用户原已手动开启 DND，睡眠期间 DND 计时继续走（DND 的过期时间基于绝对时钟），唤醒后若 DND 尚未过期则仍然有效。
- 轮询循环中，先检查 DND，再检查系统暂停，二者满足任一皆不触发提醒。

### 边界定义

仅系统级睡眠、休眠、锁屏触发暂停。单纯显示器关闭（如显示器超时熄屏、合盖后 clamshell 模式外接屏仍工作）不触发暂停。macOS 上 `NSWorkspaceScreensDidSleepNotification` 被忽略，仅响应 `NSWorkspaceWillSleepNotification` 与锁屏通知。

### 防重入

`enter_system_pause()` 在已为暂停态时返回无事；`exit_system_pause()` 在已为非暂停态时返回无事。防止合盖时先锁屏再睡眠导致的重复计时。

### 调时补偿策略

进入暂停时记录当前 `Instant`；退出暂停时计算 `elapsed = now - pause_start`，将 `water_next += elapsed` 与 `eye_next += elapsed`。不依赖 `Instant` 在睡眠期间是否自停，显式补偿确保跨平台行为一致。

## Testing Decisions

- **测试原则**：仅测外部可观测行为（进入暂停后提醒是否被抑制、退出后下次提醒时间是否正确后移），不测平台事件监听内部实现。
- **`timer.rs` 中系统暂停逻辑**：可测。通过直接操作 `TimerState` 的 `enter_system_pause` / `exit_system_pause` 方法，断言 `water_next` 与 `eye_next` 的推移量与休眠时长一致。
- **`power_monitor` 平台事件映射**：不测。其本质是系统 API 的薄封装，集成测试成本过高且依赖特定硬件状态；正确性依赖代码审查与真机验证。
- **参考**：现有 `timer.rs` 无单元测试，本 PRD 不强制要求补齐既有测试，但新增的系统暂停方法应附带测试以锁定补偿逻辑。

## Out of Scope

- 单纯显示器关闭的检测与处理（不暂停）。
- Windows 平台实现（本 PRD 聚焦 macOS；Windows 电源/锁屏检测将在后续 PRD 中覆盖）。
- 用户可开关此功能的设置项（功能恒开）。
- 托盘 UI 状态指示（静默处理）。
- 睡眠期间错过多次提醒的补偿（仅推迟下一次，不补弹）。

## Further Notes

- macOS 分布式通知 `com.apple.screenIsLocked` / `com.apple.screenIsUnlocked` 需要合适的权限，在沙盒环境中可能受限。眸润作为非沙盒桌面应用应可正常接收。若遇权限问题，可降级为仅检测睡眠事件（`NSWorkspaceWillSleepNotification`），锁屏场景由合盖睡眠覆盖大部分情况。
- Tauri v2 的 ACL 与权限系统不拦截此类原生平台代码，无需新增 capability。
- 后续 Windows 实现应保持 `power_monitor` 的接口不变，仅新增 `#[cfg(target_os = "windows")]` 的实现分支。

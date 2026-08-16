# Forza Telemetry RS — 用户手册 / User Guide

> **注意 / Note**: 英文部分由 AI 翻译，如有歧义以中文为准。The English section is AI-translated; the Chinese version takes precedence in case of ambiguity.

---

## 中文

### 简介

Forza Telemetry RS 是实时遥测工具，支持 Forza Horizon 6 与《Forza Motorsport》（2023）。通过读取游戏 UDP 数据流，提供 HUD 浮层、功率曲线分析、换挡建议、轮胎/悬挂监控、录制回放等功能。游戏格式按 UDP 包长自动识别（324 字节 = Horizon，331 字节 = Motorsport），无需手动切换。Forza Horizon 4/5 数据格式理论上与 6 一致，用户可以自行尝试，但项目未做不同版本的分隔，建议仅在 FH6 中使用。

### 系统要求

- Windows 10/11
- Forza Horizon 6 (PC) 或 Forza Motorsport (2023, PC)
- 游戏内 UDP 遥测 (Data Out) 功能已开启

### 安装与运行

1. 下载 `forza-telemetry-rs.exe`（单文件，无需 Python 或 Node.js）
2. 双击运行，主窗口自动打开
3. 首次使用需配置游戏内 Data Out（见下节）

### 游戏内设置

**Forza Horizon 6：**

1. 进入游戏 → 设置 → HUD and Gameplay
2. **Data Out**: ON
3. **IP 地址**: `127.0.0.1`
4. **端口**: `5300`（与工具的 UDP 端口一致，可在设置页修改）

**Forza Motorsport (2023)：**

1. 设置 → GAMEPLAY & HUD → UDP RACE TELEMETRY
2. **Data Out**: ON
3. **IP 地址**: `127.0.0.1`
4. **端口**: `5300`（与工具的 UDP 端口一致）
5. **格式**: 选 **CAR DASH**

设置完成后进入一辆车并起步，控制面板即开始接收数据。主窗口左下状态点悬停可查看当前识别到的游戏格式（Horizon / Motorsport）。

### 界面概览

主窗口左侧为标签栏，共 8 个页面：

| 标签 | 功能 |
|------|------|
| 总览 | HUD 预览 + 油门/刹车示波器 + 功率曲线 + G-G 图 + 胎温/悬挂/滑移 + 油门刹车柱 |
| HUD | HUD 浮层全部显示参数（字号/配色/标记线/G力浮动等） |
| 动力 | 功率/扭矩曲线 + 换挡激进程度 + 断油阈值 + 曲线管理 |
| 轮胎 | 胎温量杯 + 滑移率量杯 + 滑移阈值 |
| 悬挂 | 四轮行程示波器 + 胶囊柱状图 + 颜色阈值 |
| G值 | G-G 拖尾轨迹图 + 总G平滑 |
| 录制 | 自由录制/定时录制 + 文件管理 + 回放控制 |
| 设置 | 曲线采样参数 + 增压稳定 + EV检测 + UDP端口 |

### HUD 浮层

- 点击"启动 HUD"打开透明浮层窗口
- 浮层显示：档位、时速、RPM 条、换挡线（蓝）、断油线（红）、功率带（白）
- 爆闪提示：达到换挡点时档位和 RPM 条闪烁蓝光
- G 力浮动：加速/刹车/过弯时浮层整体漂移
- 编辑模式：点击"激活 HUD"可拖动窗口、调整大小，黄色虚线框表示编辑中
- 停车自动隐藏，起步恢复显示
- 点击"停止 HUD"关闭浮层，位置自动保存

### 换挡建议

- 自动学习当前车辆的功率曲线（EMA 采样）；仅在行驶且车轮有滑移时采样
- 在 HUD 和总览页显示换挡建议：hold（保持）/ near（接近）/ shift（换挡）/ over（超转）
- 自动检测断油转速
- 自动识别电动车（隐藏换挡/断油线；Motorsport 禁用）
- 换车时自动保存旧车曲线、加载新车曲线
- 可锁定曲线防止继续采样（双击功率图 / 按钮 / 总览车况卡片）
- 增压稳定采样：仅涡轮车有效，增压稳定后才采，避免爬升段低质量数据。动力页可调参数

### 录制与回放

- **自由录制**：点击"开始录制"，手动停止
- **定时录制**：设定倒计时 + 录制时长，自动停止
- **等有效数据**（自由录制选项）：勾选后等待遥测数据到达才开始录制
- **取消准备**：倒计时/等待数据期间按钮变为"取消准备"，可随时取消
- **尾部裁剪**：录制停止后自动裁掉末尾 N 秒
- **回放**：选择文件播放，支持暂停/继续/停止；回放结束自动恢复实时数据
- 录制与回放互斥：冲突操作（如录制中回放、回放中录制）会被拒绝并提示
- 操作失败（如没有录制文件）时底部弹出提示
- 录制文件格式为 `.bin`（原始 UDP 包按实际长度拼接，324 字节 = Horizon / 331 字节 = Motorsport，文件名以 `_fh`/`_fm` 结尾仅作标识、不参与格式判定），存放在 `tools/` 目录（打包版在 exe 同目录下）

### 参数说明

所有可调参数均配有信息提示（点击 ⓘ 图标）。悬停即显示说明，点击可锁定提示方便对照调节。详见设置页面。

### 数据文件

| 文件 | 说明 |
|------|------|
| `hud-settings.json` | 所有用户设置，删除恢复默认 |
| `hud-config.json` | HUD 窗口位置和大小 |
| `car_curves.json` | 每车功率曲线数据 |

---

## English

### Introduction

Forza Telemetry RS is a real-time telemetry tool supporting Forza Horizon 6 and Forza Motorsport (2023). It reads the game's UDP data stream and provides a HUD overlay, power curve analysis, shift advisor, tire/suspension monitoring, recording/playback, and more. The game format is auto-detected by UDP packet length (324 bytes = Horizon, 331 bytes = Motorsport) — no manual switching. The Forza Horizon 4/5 data format is theoretically identical to 6 — users may try at their own discretion, but the project does not separate game versions, so FH6 is recommended.

### Requirements

- Windows 10/11
- Forza Horizon 6 (PC) or Forza Motorsport (2023, PC)
- UDP telemetry (Data Out) enabled in-game

### Installation

1. Download `forza-telemetry-rs.exe` (single file, no dependencies required)
2. Double-click to launch — the main window opens automatically
3. Configure in-game Data Out on first use (see below)

### In-Game Setup

**Forza Horizon 6:**

1. In-game → Settings → HUD and Gameplay
2. **Data Out**: ON
3. **IP Address**: `127.0.0.1`
4. **Port**: `5300` (must match the tool's UDP port, configurable in Settings tab)

**Forza Motorsport (2023):**

1. Settings → GAMEPLAY & HUD → UDP RACE TELEMETRY
2. **Data Out**: ON
3. **IP Address**: `127.0.0.1`
4. **Port**: `5300` (must match the tool's UDP port)
5. **Format**: select **CAR DASH**

Enter a car and start driving — the control panel will begin receiving data. Hover the status dot (bottom-left) to see the detected game format (Horizon / Motorsport).

### Interface Overview

The left sidebar has 8 tabs:

| Tab | Function |
|-----|----------|
| Dashboard | HUD preview + throttle/brake oscilloscope + power curve + G-G plot + tire temp/suspension/slip + throttle/brake bars |
| HUD | Full HUD display parameters (fonts, colors, markers, G-force float, etc.) |
| Power | Power/torque curves + shift aggressiveness + limiter threshold + curve management |
| Tires | Tire temp gauges + slip ratio gauges + slip thresholds |
| Suspension | 4-wheel oscilloscopes + staggered bar chart + color thresholds |
| G-Force | G-G trail plot + total G smoothing |
| Record | Free/timed recording + file management + playback controls |
| Settings | Curve sampling + boost stable + EV detection + UDP port |

### HUD Overlay

- Click "Start HUD" to open the transparent overlay window
- Displays: gear, speed, RPM bar, shift line (blue), limiter line (red), power band (white)
- Strobe flash: gear and RPM bar flash blue when reaching shift point
- G-force float: overlay drifts with acceleration/braking/cornering
- Edit mode: click "Activate HUD" to drag and resize the window (yellow dashed border)
- Auto-hides when parked, reappears when moving
- Click "Stop HUD" to close — position is saved automatically

### Shift Advisor

- Automatically learns the current car's power curve (EMA sampling); samples only while moving with wheel slip
- Displays shift advice on HUD and Dashboard: hold / near / shift / over
- Auto-detects fuel cut RPM
- Auto-detects electric vehicles (hides shift/limiter lines; disabled for Motorsport)
- Saves curve on car change, loads saved curve on return
- Lock curve to prevent further sampling (double-click power chart / button / dashboard info card)
- Boost stable sampling: turbo cars only, waits for boost to stabilize before sampling. Parameters on Power page

### Recording & Playback

- **Free Record**: click "Start Recording", stop manually
- **Timed Record**: set countdown + duration, auto-stop
- **Wait for Data** (free record option): waits for telemetry data before starting
- **Cancel Pending**: during countdown/data wait the button becomes "Cancel", cancel anytime
- **Tail Trim**: auto-trim last N seconds after recording stops
- **Playback**: select a file to play, with pause/resume/stop; live data resumes automatically after playback ends
- Recording and playback are mutually exclusive: conflicts (e.g. playback while recording) are rejected with a notice
- Failed operations (e.g. no recordings) show a toast notice at the bottom
- Recordings saved as `.bin` files (raw UDP packets at their actual length: 324 bytes = Horizon, 331 bytes = Motorsport; the `_fh`/`_fm` filename suffix is for identification only and does not affect format detection) in `tools/` (next to the exe in the packaged build)

### Parameters

All adjustable parameters have info tooltips (click the ⓘ icon). Hover for instant display, click to lock the tooltip for reference while adjusting.

### Data Files

| File | Description |
|------|-------------|
| `hud-settings.json` | All user settings; delete to reset defaults |
| `hud-config.json` | HUD window position and size |
| `car_curves.json` | Per-car power curve data |

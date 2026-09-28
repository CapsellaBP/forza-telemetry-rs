# Forza Telemetry RS

> **v0.3.0** — Forza 实时遥测工具 (2026-08-16)

实时遥测工具：透明 HUD 浮层、换挡建议、功率曲线分析、轮胎/悬挂监控、录制与回放。支持 Forza Horizon 6 与 Forza Motorsport (2023)。Forza Horizon 4/5 数据格式理论上与 6 一致，用户可以自行尝试，但项目未做不同版本的分隔，建议仅在 FH6 中使用。

*A real-time telemetry tool: transparent HUD overlay, shift advisor, power curve analysis, tire/suspension monitoring, recording & playback. Supports Forza Horizon 6 and Forza Motorsport (2023). The Forza Horizon 4/5 data format is theoretically identical to 6 — users may try at their own discretion, but the project does not separate game versions, so FH6 is recommended.*

## 功能 / Features

- **HUD 透明浮层 / HUD overlay** — 档位、时速、RPM、换挡线、最大转速线、功率带，可拖动缩放 / gear, speed, RPM, shift line, max RPM line, power band, draggable & resizable
- **换挡建议 / Shift advisor** — 自动学习每车功率曲线，提示 hold / near / shift / over / auto-learns each car's power curve with hold/near/shift/over advice
- **功率曲线分析 / Power analysis** — 扭矩与功率双线图、最大转速检测、换挡点计算 / torque & power curves, max RPM detection, shift-point calculation
- **轮胎与悬挂 / Tires & suspension** — 胎温、滑移率、悬挂行程实时监控 / real-time tire temp, slip ratio and suspension travel
- **G 力 / G-force plot** — 横向/纵向加速度拖尾轨迹图 / lateral & longitudinal acceleration trail plot
- **录制与回放 / Recording & playback** — 原始 UDP 包录制，按时间戳对齐回放 / raw UDP capture with timestamp-aligned playback
- **每车记忆 / Per-car memory** — 换车自动保存与加载曲线 / curves auto-saved and reloaded per car

## 快速开始 / Quick Start

要求 / Requires: Windows 10/11 + Forza Horizon 6 或 Forza Motorsport (2023, PC)。

1. 下载最新 [release](https://github.com/CapsellaBP/forza-telemetry-rs/releases) 中的 `forza-telemetry-rs.exe`（单文件，无需 Python 或 Node.js）/ Download `forza-telemetry-rs.exe` from the latest release (single file, no dependencies)
2. 游戏内开启 UDP Data Out：IP `127.0.0.1`，端口 `5300`（Motorsport 需在格式里选 CAR DASH）/ In-game: enable UDP Data Out with IP `127.0.0.1`, port `5300` (select CAR DASH format in Motorsport)
3. 双击运行，进车开起来即可 / Double-click to run, then just drive

详见[用户手册](user-guide.md)。*See the user guide for details.*

## 文档 / Documentation

| 文档 / Doc | 语言 / Language |
|------|------|
| 用户手册 / [User Guide](user-guide.md) | 中英双语 / ZH & EN |
| 设置参数 / [Settings](settings.md) | 中文 / ZH |
| 调校指南 / [Tuning Guide](tuning.md) | 中文 / ZH |
| 遥测格式 / [Telemetry Format](telemetry.md) | 中文 / ZH |
| 开发指南 / [Development](development.md) | 中文 / ZH |

## 构建 / Building

```bash
npm install
npm run tauri build
# 输出 / Output: src-tauri/target/release/forza-telemetry-rs.exe
```

# 开发指南

## 环境配置

| 工具 | 版本 | 说明 |
|---|---|---|
| Rust | 1.96+ | `https://rustup.rs` |
| Node.js | 24+ | `https://nodejs.org` |
| VS 2022 Build Tools | 17+ | MSVC 链接器 (Tauri 需要) |

### VS Code 插件

- **Vue - Official** (Volar) — Vue 3 语法高亮 + 类型检查
- **rust-analyzer** — Rust 智能提示

## 日常开发

```bash
cargo tauri dev           # 启动开发模式 (编译 Rust + Vite + Tauri 窗口)
npm run dev               # 仅前端 (http://localhost:1420)
cargo build --manifest-path src-tauri/Cargo.toml   # 仅检查 Rust
cargo test --manifest-path src-tauri/Cargo.toml --lib  # Rust 测试
npx vue-tsc --noEmit      # TypeScript 类型检查
python tools/capture_raw.py 10   # 抓真实数据 (10 秒)
```

## 架构

```
┌─ Tauri App ───────────────────────────────────────┐
│                                                    │
│  Rust 后端 (server.rs)                              │
│  ┌─ UDP 收包 → telemetry::parse()                 │
│  │  → 换车检测 → 档位防抖 → EV → ShiftAdvisor    │
│  │  → State.raw                                    │
│  ├─ 广播循环 62.5Hz                                │
│  │  → build_broadcast() → WebSocket :9000          │
│  │  → 回放: 读 mpsc channel → 本地 buffer → feed  │
│  ├─ 设置变更 → State.settings → hud-settings.json │
│  └─ mpsc channel ← WebSocket 消息                  │
│       → lib.rs 后台线程 → Tauri 命令                │
│                                                    │
│  Vue 3 前端 (8 标签页)                              │
│  ┌─ ws://127.0.0.1:9000                            │
│  │  → telemetry ref → 所有组件响应式更新            │
│  └─ 用户操作 → WebSocket → 服务端                   │
│                                                    │
│  HUD (透明 WebView2)                                │
│  ┌─ ws://127.0.0.1:9000                            │
│  └─ 档位/速度/RPM条/标记线/G力浮动                  │
└────────────────────────────────────────────────────┘
```

### 数据流

1. 游戏每帧发 UDP（Horizon 324 字节 / Motorsport 331 字节，按包长自动分派）→ `telemetry::parse()` → `State.raw`
2. 广播循环 16ms 读取 → `build_broadcast()` → JSON → 所有 WS 客户端
3. 前端 `onmessage` → `telemetry` ref → 组件响应式更新
4. 用户调滑块 → `{"action":"set_setting","payload":{...}}` → 更新 `State.settings` → 下次广播生效 → 写 `hud-settings.json`

### WS 客户端管理（防内存泄漏）

- 每连接使用**有界队列**（容量 4）+ `try_send`：慢/死客户端丢帧而非无限积压
- 广播循环每 tick 清理已关闭连接（`retain(!is_closed)`）
- 10 秒 Ping 心跳 + TCP keepalive（30s/10s）检测半开连接
- 客户端上限 8，超出丢弃最旧
- 广播间隔使用 `MissedTickBehavior::Skip`

### 回放架构

```
前端 "播放" → WS → server.rs → mpsc channel → lib.rs
  → start_playback() 读 .bin → mpsc channel(原始字节)
    → 广播循环 try_recv → pb_buf (本地 buffer)
      → 每 tick 推进一帧 (按 timestamp_ms 真实间隔)
        → telemetry::parse() → State.raw → 换车检测 → advisor.feed()
          → build_broadcast() → WebSocket
```

回放数据通过 `std::sync::mpsc::channel<Vec<u8>>` 从 lib.rs 直投到广播循环，不依赖 Arc 跨线程共享。

### Rust 模块

| 文件 | 职责 |
|---|---|
| `main.rs` | 入口，隐藏 Windows 控制台 |
| `lib.rs` | Tauri 命令、HUD 窗口、录制/回放、HUD 配置持久化 |
| `server.rs` | UDP 监听、WebSocket 广播、设置管理、回放推进、EV 检测 |
| `telemetry.rs` | FH 324B / Motorsport 331B Sled+Dash 解析（按包长分派）+ detect_pkt_size + 测试 |
| `shift.rs` | 换挡引擎：EMA 采样、峰值检测、断油识别、增压稳定过滤 |

### 录制/回放命令

| 命令 | 说明 |
|---|---|
| `start_record` | 立即录制 (自由模式) |
| `start_record {delay,duration,trim}` | 定时录制 |
| `stop_record` | 停止录制 + 尾部裁剪 |
| `start_playback` | 回放最新录制 |
| `playback_file {path}` | 指定文件回放 |
| `stop_playback` / `pause_playback` / `resume_playback` | 回放控制 |

### 测试

```bash
cargo test --manifest-path src-tauri/Cargo.toml --lib
# 输出: "Parsed 565 real packets OK"

python tools/capture_raw.py 10  # 先抓包
cargo test -- --nocapture       # 集成测试
```

## 配置文件

| 文件 | 说明 |
|---|---|
| `hud-settings.json` | 所有用户设置 (自动生成, gitignore) |
| `hud-config.json` | HUD 窗口位置/大小/编辑状态 |
| `car_curves.json` | 每车功率曲线数据 (换车时自动保存) |

## 打包发布

```bash
cargo tauri build
# 输出 src-tauri/target/release/forza-telemetry-rs.exe，无需 Python/Node.js
```

> **注意**：必须通过 `cargo tauri build`（或 `npm run tauri build`）构建。裸 `cargo build --release` 产物未内嵌前端资源，会指向 devUrl 而无法独立运行。构建前需关闭正在运行的 exe（Windows 锁定文件）。

### WebView2 软件渲染（勿删配置）

`tauri.conf.json` 的 `additionalBrowserArgs: "--disable-gpu ..."` 配置 WebView2 软件渲染。HUD 窗口经 lib.rs 从主窗口配置继承同参数。勿当作无用配置清理。

修改 `tauri.conf.json` 后若 `cargo check` 秒回 Finished（增量指纹未触发重编译），用 `cargo clean -p forza-telemetry-rs` 强制重编译验证配置。

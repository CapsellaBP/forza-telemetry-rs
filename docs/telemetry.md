# 遥测数据格式

## UDP 数据包结构

按包长自动识别两种格式（均为 little-endian，float 为 4 字节单精度）：

### Horizon 家族 (FH4 / FH5 / FH6) — 324 字节

| 段 | 字节偏移 | 大小 | 内容 |
|----|---------|------|------|
| Sled | 0-231 | 232B | 核心车辆物理数据 |
| Horizon Ext | 232-243 | 12B | Horizon 专属扩展 |
| Dash | 244-322 | 79B | 比赛/仪表数据 |
| Padding | 323 | 1B | 填充 |

### Motorsport 家族 (Forza Motorsport 2023, Car Dash) — 331 字节

| 段 | 字节偏移 | 大小 | 内容 |
|----|---------|------|------|
| Sled | 0-231 | 232B | 与 Horizon 完全同构 |
| Dash | 232-310 | 79B | 与 Horizon Dash 字段同构 (偏移减 12) |
| 扩展 | 311-330 | 20B | 胎损 4×f32 + 赛道序号 i32 |

工具录放按**包内时间戳**驱动（回放按时间戳对齐推进，trim 按时间戳二分裁剪），不依赖固定发包速率。

## Sled 段关键字段

| 偏移 | 类型 | 字段 | 说明 |
|------|------|------|------|
| 0 | s32 | is_race_on | 1=比赛中 |
| 8 | f32 | engine_max_rpm | 最大转速 |
| 12 | f32 | engine_idle_rpm | 怠速转速 |
| 16 | f32 | current_engine_rpm | 当前转速 |
| 20 | f32 | accel_x | 横向加速度 (m/s²) |
| 24 | f32 | accel_y | 垂向加速度 |
| 28 | f32 | accel_z | 纵向加速度 |
| 32-40 | f32×3 | velocity | 速度分量 (m/s) |
| 56-64 | f32×3 | yaw/pitch/roll | 姿态角 (rad) |
| 68-80 | f32×4 | susp_travel | 悬挂行程 (归一化 0~1) |
| 84-96 | f32×4 | tire_slip_ratio | 轮胎滑移率 (0=全抓地) |
| 100-112 | f32×4 | wheel_rot_speed | 轮速 (rad/s) |
| 164-176 | f32×4 | tire_slip_angle | 滑移角 |
| 180-192 | f32×4 | tire_combined_slip | 组合滑移 |
| 196-208 | f32×4 | susp_travel_meters | 悬挂行程 (米) |
| 212 | s32 | car_ordinal | 车辆唯一 ID |
| 216 | s32 | car_class | PI 等级 (0=D~7=X) |
| 220 | s32 | car_perf_index | PI 数值 |
| 224 | s32 | drivetrain_type | 0=FWD 1=RWD 2=AWD |
| 228 | s32 | num_cylinders | 气缸数 |

## Dash 段关键字段

Horizon (244 起) 与 Motorsport (232 起) 逐字段同构，偏移恰差 12（Horizon 头长度）：

| 偏移 (Horizon) | 偏移 (Motorsport) | 类型 | 字段 | 说明 |
|------|------|------|------|------|
| 244 | 232 | f32 | position_x | 世界坐标 X |
| 248 | 236 | f32 | position_y | 世界坐标 Y |
| 252 | 240 | f32 | position_z | 世界坐标 Z |
| 256 | 244 | f32 | speed | 速度 (m/s) |
| 260 | 248 | f32 | power | 功率 (瓦特) |
| 264 | 252 | f32 | torque | 扭矩 (N·m) |
| 268-280 | 256-268 | f32×4 | tire_temp | 胎温 (°F 原始值) |
| 284 | 272 | f32 | boost | 涡轮增压 (psi) |
| 288 | 276 | f32 | fuel | 剩余油量 (比例) |
| 292 | 280 | f32 | distance | 总里程 (m) |
| 296 | 284 | f32 | best_lap | 最佳圈速 (秒) |
| 300 | 288 | f32 | last_lap | 上一圈速 (秒) |
| 304 | 292 | f32 | current_lap | 当前圈速 (秒) |
| 308 | 296 | f32 | race_time | 比赛时间 (秒) |
| 312 | 300 | u16 | lap | 当前圈数 |
| 314 | 302 | u8 | race_pos | 比赛排名 |
| 315 | 303 | u8 | accel | 油门 (0-255) |
| 316 | 304 | u8 | brake | 刹车 (0-255) |
| 317 | 305 | u8 | clutch | 离合 (0-255) |
| 318 | 306 | u8 | handbrake | 手刹 (0-255) |
| 319 | 307 | u8 | gear | 档位 (编码见下) |
| 320 | 308 | s8 | steer | 转向 (-128~127) |
| 321 | 309 | u8 | normal_driving_line | 行车线辅助 |
| 322 | 310 | u8 | normal_ai_brake_diff | AI 刹车差异 |

## Motorsport 扩展段 (仅 331 字节包)

| 偏移 | 类型 | 字段 | 说明 |
|------|------|------|------|
| 311 | f32 | tire_wear_fl | 前左胎损 |
| 315 | f32 | tire_wear_fr | 前右胎损 |
| 319 | f32 | tire_wear_rl | 后左胎损 |
| 323 | f32 | tire_wear_rr | 后右胎损 |
| 327 | i32 | track_ordinal | 赛道序号 |

## 档位编码

Horizon 与 Motorsport 家族编码相同：
- `0` = 倒挡 (R)，`1`-`10` = 1-10 挡
- `11` = 换挡过程标志，服务端过滤，HUD 沿用上一稳定档位
- 前端将 `-1` 显示为 N

## 检测游戏格式

根据 UDP 包大小识别：
- **324 字节** → Horizon 家族
- **331 字节** → Forza Motorsport 2023 (Car Dash)

录制文件同样按内容检测：大小 `%331==0 → Motorsport`，`%324==0 → Horizon`（331 优先，文件名不参与解析）。

// Shift Advisor — EMA power curve sampling + fuel cut detection + EV detection
use std::collections::HashMap;

const RPM_BIN_SIZE: i32 = 100;

pub struct ShiftAdvisor {
    bins: HashMap<i32, (u32, f64, f64)>, // rpm_bin → (count, ema_torque, ema_power)
    pub(crate) sample_count: u32,
    pub(crate) ready: bool,
    peak_power_rpm: f64,
    pub(crate) locked: bool,

    // Settings (pub for server access)
    pub(crate) aggressiveness_pct: i32,
    pub(crate) limiter_threshold: f64,
    pub(crate) shift_trigger: f64,
    pub(crate) alpha: f64,
    pub(crate) throttle_min: f64,
    pub(crate) skip_ms: u32,
    pub(crate) power_drop_limit: f64,
    pub(crate) boost_stable_sample: bool,
    pub(crate) boost_stable_ms: u32,
    pub(crate) boost_stable_tol: f64,
    last_boost: f64,
    pub(crate) stable_value: f64,
    stable_value_set: bool,
    steady_since: Option<std::time::Instant>,
    boost_stable: bool,

    // Per-frame state
    skip_until: Option<std::time::Instant>,
    last_gear_sample: Option<i32>,
    last_rpm_sample: f64,
    pub(crate) idle_rpm: f64,
    pub(crate) rpm_max: f64,
    last_urgency: String,
    hysteresis_timer: u32,
    last_gear_adv: Option<i32>,

    // Fuel cut detection
    fuel_cut_rpm: f64,
    fuel_cut_lo: f64,
    fc_peak: f64,
    fc_valleys: Vec<f64>,
    last_rpm_tick: f64,
    last_throttle: f64,
    last_gear_fc: Option<i32>,
    fc_rising: bool,
}

impl Default for ShiftAdvisor {
    fn default() -> Self {
        Self {
            bins: HashMap::new(), sample_count: 0, ready: false, peak_power_rpm: 0.0, locked: false,
            aggressiveness_pct: 0, limiter_threshold: 1.0, shift_trigger: 1.0,
            alpha: 0.25, throttle_min: 0.95, skip_ms: 130, power_drop_limit: 0.0,
            boost_stable_sample: false, boost_stable_ms: 500, boost_stable_tol: 0.90,
            last_boost: 0.0, stable_value: 0.0, stable_value_set: false, steady_since: None, boost_stable: false,
            skip_until: None, last_gear_sample: None, last_rpm_sample: 0.0,
            idle_rpm: 1500.0, rpm_max: 8000.0, last_urgency: "hold".into(),
            hysteresis_timer: 0, last_gear_adv: None,
            fuel_cut_rpm: 0.0, fuel_cut_lo: 0.0, fc_peak: 0.0,
            fc_valleys: Vec::new(),
            last_rpm_tick: 0.0, last_throttle: 0.0, last_gear_fc: None, fc_rising: true,
        }
    }
}

fn bin(rpm: f64) -> i32 { (rpm as i32 / RPM_BIN_SIZE) * RPM_BIN_SIZE }

impl ShiftAdvisor {
    pub fn new() -> Self { Self::default() }

    pub fn reset(&mut self) {
        self.bins.clear(); self.sample_count = 0; self.peak_power_rpm = 0.0; self.ready = false;
        self.fuel_cut_rpm = 0.0; self.fuel_cut_lo = 0.0; self.fc_valleys.clear();
        self.fc_peak = 0.0;
        self.stable_value = 0.0; self.stable_value_set = false;
        self.last_boost = 0.0; self.steady_since = None; self.boost_stable = false;
    }

    pub fn to_dict(&self) -> serde_json::Value {
        let mut bins_map = serde_json::Map::new();
        for (k, v) in &self.bins {
            bins_map.insert(k.to_string(), serde_json::json!([v.0, v.1, v.2]));
        }
        serde_json::json!({
            "bins": bins_map, "sample_count": self.sample_count,
            "peak_power_rpm": self.peak_power_rpm,
            "fuel_cut_rpm": self.fuel_cut_rpm, "fuel_cut_lo": self.fuel_cut_lo,
            "idle_rpm": self.idle_rpm, "rpm_max": self.rpm_max, "ready": self.ready,
            "locked": self.locked, "stable_value": self.stable_value,
            "stable_value_set": self.stable_value_set,
        })
    }

    pub fn from_dict(d: &serde_json::Value) -> Self {
        let mut obj = Self::new();
        obj.locked = d.get("locked").and_then(|v| v.as_bool()).unwrap_or(false);
        if let Some(bins) = d.get("bins").and_then(|b| b.as_object()) {
            for (k, v) in bins {
                if let (Ok(rpm), Some(arr)) = (k.parse::<i32>(), v.as_array()) {
                    if arr.len() == 3 {
                        obj.bins.insert(rpm, (
                            arr[0].as_u64().unwrap_or(0) as u32,
                            arr[1].as_f64().unwrap_or(0.0),
                            arr[2].as_f64().unwrap_or(0.0),
                        ));
                    }
                }
            }
        }
        obj.sample_count = d.get("sample_count").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
        obj.peak_power_rpm = d.get("peak_power_rpm").and_then(|v| v.as_f64()).unwrap_or(0.0);
        obj.fuel_cut_rpm = d.get("fuel_cut_rpm").and_then(|v| v.as_f64()).unwrap_or(0.0);
        obj.fuel_cut_lo = d.get("fuel_cut_lo").and_then(|v| v.as_f64()).unwrap_or(0.0);
        obj.idle_rpm = d.get("idle_rpm").and_then(|v| v.as_f64()).unwrap_or(1500.0);
        obj.rpm_max = d.get("rpm_max").and_then(|v| v.as_f64()).unwrap_or(8000.0);
        obj.ready = d.get("ready").and_then(|v| v.as_bool()).unwrap_or(false);
        if let Some(v) = d.get("stable_value").and_then(|v| v.as_f64()) { obj.stable_value = v; }
        obj.stable_value_set = d.get("stable_value_set").and_then(|v| v.as_bool()).unwrap_or(false);
        obj
    }

    pub fn get_curve(&self, band_pct: f64) -> serde_json::Value {
        let mut rpm_keys: Vec<i32> = self.bins.keys().copied().collect();
        rpm_keys.sort();
        let mut torque_pts = Vec::new();
        let mut power_pts = Vec::new();
        for b in &rpm_keys {
            if let Some((cnt, tq, pw)) = self.bins.get(b) {
                if *cnt >= 3 {
                    torque_pts.push(vec![serde_json::Value::from(*b as f64), serde_json::Value::from((tq * 10.0).round() / 10.0)]);
                    power_pts.push(vec![serde_json::Value::from(*b as f64), serde_json::Value::from((pw / 1000.0 * 10.0).round() / 10.0)]);
                }
            }
        }

        let fc_limit = if self.fuel_cut_rpm > 0.0 { self.fuel_cut_rpm } else if self.fuel_cut_lo > 0.0 { self.fuel_cut_lo } else { self.rpm_max };
        let mut band_lo = 0.0;
        let mut band_hi = 0.0;
        if self.ready && !power_pts.is_empty() {
            let peak_kw: f64 = power_pts.iter()
                .filter(|p| p[0].as_f64().unwrap_or(0.0) < fc_limit)
                .map(|p| p[1].as_f64().unwrap_or(0.0))
                .fold(0.0, f64::max);
            let threshold = peak_kw * band_pct;
            let in_band: Vec<f64> = power_pts.iter()
                .filter(|p| p[1].as_f64().unwrap_or(0.0) >= threshold && p[0].as_f64().unwrap_or(0.0) < fc_limit)
                .map(|p| p[0].as_f64().unwrap_or(0.0))
                .collect();
            if !in_band.is_empty() {
                band_lo = in_band.iter().fold(f64::MAX, |a, &b| a.min(b));
                band_hi = in_band.iter().fold(f64::MIN, |a, &b| a.max(b));
            }
        }

        serde_json::json!({
            "torque": torque_pts, "power_kw": power_pts,
            "optimal_rpm": self.calc_optimal(),
            "peak_power_rpm": self.peak_power_rpm,
            "fuel_cut_rpm": self.fuel_cut_rpm,
            "power_band_lo": band_lo, "power_band_hi": band_hi,
            "ready": self.ready, "samples": self.sample_count,
            "locked": self.locked, "boost_stable": self.boost_stable,
            "stable_value": self.stable_value, "stable_value_set": self.stable_value_set,
        })
    }

    fn calc_optimal(&self) -> f64 {
        if !self.ready { return self.rpm_max * 0.90; }
        let range = self.rpm_max - self.idle_rpm;
        let offset = self.aggressiveness_pct as f64 / 100.0 * range;
        (self.idle_rpm + 500.0).max((self.rpm_max - 200.0).min(self.peak_power_rpm + offset))
    }

    pub fn feed(&mut self, rpm: f64, torque: f64, power: f64, throttle: f64, gear: i32, boost: f64, speed: f64, slip: f64) {
        if self.locked { return; }

        self.idle_rpm = self.idle_rpm.max(500.0);
        self.rpm_max = self.rpm_max.max(1000.0);

        // Boost stability: detect sustained level via exact frame-to-frame match,
        // confirmed by wall-clock duration (frame-rate independent)
        if self.boost_stable_sample {
            let same = (boost * 10.0).round() == (self.last_boost * 10.0).round();
            if same {
                if self.steady_since.is_none() { self.steady_since = Some(std::time::Instant::now()); }
                if self.steady_since.is_some_and(|t| t.elapsed() >= std::time::Duration::from_millis(self.boost_stable_ms as u64))
                    && boost >= 0.0 && speed > 0.0 {
                    self.stable_value = (boost * 10.0).round() / 10.0;
                    self.stable_value_set = true;
                }
            } else {
                self.steady_since = None;
            }
            // Pass if boost matches established baseline
            self.boost_stable = boost >= self.stable_value * self.boost_stable_tol;
            self.last_boost = boost;
        } else {
            self.boost_stable = true;
        }

        // Fuel cut detection
        let near_limiter = rpm > self.rpm_max * 0.85;
        if near_limiter && throttle >= self.throttle_min && Some(gear) == self.last_gear_fc {
            if rpm > self.last_rpm_tick {
                self.fc_rising = true;
                self.fc_peak = rpm;
            } else if self.fc_rising && self.last_rpm_tick - rpm > 80.0 {
                // Fuel cut kills combustion: torque flips negative (engine
                // braking). Kerb/load transients jiggle rpm the same way but
                // torque stays positive at full throttle — reject those so
                // they can't drag the limiter line left.
                if torque < 0.0 {
                    self.fc_valleys.push(rpm);
                    if self.fc_valleys.len() > 30 { self.fc_valleys.remove(0); }
                    let mut valleys = self.fc_valleys.clone(); valleys.sort_by(|a,b| a.partial_cmp(b).unwrap());
                    self.fuel_cut_lo = valleys[valleys.len()/2];
                    // The limiter is a hard ceiling per car: track the highest
                    // confirmed cut. Never moves down — a lower peak carries no
                    // information (the cut still happened), a higher one
                    // disproves the old value and snaps the line right at once.
                    self.fuel_cut_rpm = self.fuel_cut_rpm.max(self.fc_peak);
                }
                self.fc_rising = false;
            }
        }
        self.last_rpm_tick = rpm;
        self.last_throttle = throttle;
        self.last_gear_fc = Some(gear);

        // Shift skip — wall-clock ms, frame-rate independent (FH6 61Hz vs
        // FM8 ~124Hz packets both reject for the same real duration)
        if Some(gear) != self.last_gear_sample {
            self.last_gear_sample = Some(gear);
            self.skip_until = Some(std::time::Instant::now() + std::time::Duration::from_millis(self.skip_ms as u64));
        }
        let skipping = self.skip_until.is_some_and(|t| std::time::Instant::now() < t);

        let rpm_rising = rpm >= self.last_rpm_sample - 20.0;

        // FM8 countdown revving (speed 0, wheels not transmitting) must not feed the power curve
        if rpm >= 500.0 && throttle >= self.throttle_min && rpm_rising
            && !skipping
            && self.boost_stable
            && speed > 0.0 && slip > 0.0
        {
            let b = bin(rpm);
            if let Some((cnt, ema_tq, ema_pw)) = self.bins.get_mut(&b) {
                if *cnt >= 3 && *ema_pw > 0.0 && power < *ema_pw * self.power_drop_limit {
                    // Reject dirty data
                } else {
                    let a = self.alpha;
                    *ema_tq = *ema_tq * (1.0 - a) + torque * a;
                    *ema_pw = *ema_pw * (1.0 - a) + power * a;
                    *cnt += 1;
                    self.sample_count += 1;
                    if self.sample_count % 60 == 0 { self.recalc(); }
                }
            } else {
                self.bins.insert(b, (1, torque, power));
                self.sample_count += 1;
            }
            self.last_rpm_sample = rpm;
        } else {
            self.last_rpm_sample = rpm;
        }
    }

    fn recalc(&mut self) {
        if self.bins.len() < 5 { return; }
        let fc_limit = if self.fuel_cut_rpm > 0.0 { self.fuel_cut_rpm } else if self.fuel_cut_lo > 0.0 { self.fuel_cut_lo } else { self.rpm_max };
        let mut best_rpm = 0;
        let mut best_pw = 0.0f64;
        for (b, (cnt, _, pw)) in &self.bins {
            if *b as f64 >= fc_limit || *cnt < 3 { continue; }
            if *pw > best_pw { best_pw = *pw; best_rpm = *b; }
        }
        if best_rpm > 0 { self.peak_power_rpm = best_rpm as f64; self.ready = true; }
    }

    pub fn advice(&mut self, rpm: f64, rpm_max: f64, throttle: f64, gear: i32) -> serde_json::Value {
        if Some(gear) != self.last_gear_adv {
            self.last_urgency = "hold".into();
            self.hysteresis_timer = 0;
            self.last_gear_adv = Some(gear);
        }

        let limiter_rpm = if self.fuel_cut_rpm > 0.0 { self.fuel_cut_rpm }
            else { (rpm_max * self.limiter_threshold).round() };
        let limiter_zone = limiter_rpm > 0.0 && rpm > limiter_rpm * 0.99;

        let (optimal, pct, urgency) = if !self.ready {
            let opt = rpm_max * 0.90;
            (opt, if opt > 0.0 { rpm / opt } else { 0.0 }, "hold")
        } else {
            let opt = self.calc_optimal();
            let pct = if opt > 0.0 { rpm / opt } else { 0.0 };
            let trigger = self.shift_trigger;
            let urg = if throttle < 0.35 { "hold" }
                else if pct < trigger - 0.10 { "hold" }
                else if pct < trigger { "near" }
                else if pct < trigger + 0.10 { "shift" }
                else { "over" };
            (opt, pct, urg)
        };

        let mut urgency = urgency.to_string();
        if limiter_zone && matches!(urgency.as_str(), "shift" | "over" | "near") {
            urgency = "over".into();
        }

        // Hysteresis
        if matches!(self.last_urgency.as_str(), "over" | "shift") && urgency == "hold" {
            self.hysteresis_timer += 1;
            if self.hysteresis_timer < 15 { urgency = self.last_urgency.clone(); }
            else { self.hysteresis_timer = 0; }
        } else { self.hysteresis_timer = 0; }
        self.last_urgency = urgency.clone();

        serde_json::json!({
            "shift_rpm": (optimal as f64).round(),
            "peak_power_rpm": self.peak_power_rpm.round(),
            "limiter_rpm": limiter_rpm.round(),
            "fuel_cut_rpm": self.fuel_cut_rpm.round(),
            "urgency": urgency,
            "pct_to_shift": (pct * 1000.0).round() / 1000.0,
            "ready": self.ready,
            "samples": self.sample_count,
            "limiter": limiter_zone,
        })
    }
}

/// EV detection: if we see > 2 distinct forward gears, it's ICE, not EV
pub fn check_ev(gears_seen: &mut Vec<i32>, gear: i32, throttle: f64, check_count: &mut u32, limit: u32, is_ev: &mut bool) {
    if *is_ev && gear >= 1 { gears_seen.push(gear); }
    // If we ever see a gear > 2, it's ICE
    if *is_ev && gear > 2 { *is_ev = false; }

    if !*is_ev && *check_count < limit {
        if throttle > 0.8 {
            if gear >= 1 && !gears_seen.contains(&gear) { gears_seen.push(gear); }
            *check_count += 1;
        }
        if *check_count >= limit {
            let forward: Vec<i32> = gears_seen.iter().filter(|&&g| g >= 1).copied().collect();
            *is_ev = forward.len() <= 2;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn feed_n(adv: &mut ShiftAdvisor, n: u32, speed: f64, slip: f64) {
        for _ in 0..n {
            adv.feed(5000.0, 400.0, 200000.0, 1.0, 3, 0.0, speed, slip);
        }
    }

    #[test]
    fn power_sampling_requires_motion_and_slip() {
        let mut a = ShiftAdvisor::new();
        a.skip_ms = 0; // this test gates on motion/slip, not the shift window
        feed_n(&mut a, 20, 0.0, 0.5);    // countdown revving: stationary
        assert_eq!(a.sample_count, 0);
        feed_n(&mut a, 20, 30.0, 0.0);   // moving but wheels not transmitting
        assert_eq!(a.sample_count, 0);
        feed_n(&mut a, 20, 30.0, 0.05);  // normal driving
        assert!(a.sample_count > 0);
    }

    #[test]
    fn shift_skip_ms_blocks_then_allows_sampling() {
        let mut a = ShiftAdvisor::new();
        a.skip_ms = 50;
        // First feed on a gear arms the skip window — rejected inside it
        feed_n(&mut a, 5, 30.0, 0.05);
        assert_eq!(a.sample_count, 0);
        // A gear change re-arms the window
        a.feed(5000.0, 400.0, 200000.0, 1.0, 4, 0.0, 30.0, 0.05);
        assert_eq!(a.sample_count, 0);
        // Past the window, sampling resumes (same gear — no re-arm)
        std::thread::sleep(std::time::Duration::from_millis(60));
        for _ in 0..3 {
            a.feed(5000.0, 400.0, 200000.0, 1.0, 4, 0.0, 30.0, 0.05);
        }
        assert!(a.sample_count > 0);
    }

    /// Feed one limiter-bounce-shaped rpm pattern: rise to `peak`, then a
    /// >80 rpm single-frame drop with the given torque on the drop frame.
    fn feed_bounce(adv: &mut ShiftAdvisor, peak: f64, drop_torque: f64) {
        for r in [peak - 300.0, peak - 150.0, peak] {
            adv.feed(r, 500.0, 300000.0, 1.0, 3, 0.0, 30.0, 0.05);
        }
        adv.feed(peak - 120.0, drop_torque, 100000.0, 1.0, 3, 0.0, 30.0, 0.05);
    }

    #[test]
    fn fuel_cut_rejects_positive_torque_transients() {
        let mut a = ShiftAdvisor::new();
        // Kerb-style rpm jiggle: same rise/drop shape, torque stays positive
        feed_bounce(&mut a, 7900.0, 400.0);
        feed_bounce(&mut a, 7850.0, 450.0);
        assert_eq!(a.fuel_cut_rpm, 0.0);
        // Real fuel cut: torque flips negative on the drop frame
        feed_bounce(&mut a, 7900.0, -200.0);
        assert_eq!(a.fuel_cut_rpm, 7900.0);
    }

    #[test]
    fn fuel_cut_snaps_up_and_never_back_down() {
        let mut a = ShiftAdvisor::new();
        feed_bounce(&mut a, 7800.0, -200.0);
        feed_bounce(&mut a, 7820.0, -200.0);
        assert_eq!(a.fuel_cut_rpm, 7820.0);
        // One confirmed bounce at a higher ceiling pulls the line right at once
        feed_bounce(&mut a, 7900.0, -200.0);
        assert_eq!(a.fuel_cut_rpm, 7900.0);
        // Lower confirmed peaks never drag it back down
        feed_bounce(&mut a, 7810.0, -200.0);
        assert_eq!(a.fuel_cut_rpm, 7900.0);
    }

    #[test]
    fn boost_stable_ms_confirms_after_window() {
        let mut a = ShiftAdvisor::new();
        a.skip_ms = 0;
        a.boost_stable_sample = true;
        a.boost_stable_ms = 50;
        // Steady boost inside the window — baseline not established yet
        feed_n(&mut a, 3, 30.0, 0.05);
        assert!(!a.stable_value_set);
        // Past the window — baseline confirmed
        std::thread::sleep(std::time::Duration::from_millis(60));
        for _ in 0..3 {
            a.feed(5000.0, 400.0, 200000.0, 1.0, 3, 0.0, 30.0, 0.05);
        }
        assert!(a.stable_value_set);
    }
}

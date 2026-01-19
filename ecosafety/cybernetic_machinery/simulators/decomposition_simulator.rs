use std::f64::consts::E;

#[derive(Clone, Copy, Debug)]
pub struct DecompositionParams {
    /// Initial contaminant concentration in tray material or leachate (mg/kg or mg/L)
    pub c0: f64,
    /// First-order decay constant (1/day)
    pub k: f64,
    /// Simulation horizon (days)
    pub t_max_days: u32,
    /// Toxicity corridor limit (mg/kg or mg/L)
    pub tox_limit: f64,
    /// Target day by which concentration must fall below tox_limit (days)
    pub corridor_day: u32,
    /// Target biodegradation fraction by t_max (fraction 0–1, e.g. 0.90 for 90%)
    pub target_deg_fraction: f64,
}

#[derive(Clone, Copy, Debug)]
pub struct DecompositionState {
    /// Concentration at final simulated day
    pub c_final: f64,
    /// Fraction degraded at final day (0–1)
    pub frac_degraded_final: f64,
    /// First day on which C(t) <= tox_limit (if reached within horizon)
    pub day_first_safe: Option<u32>,
    /// Normalized toxicity risk coordinate r_tox in [0, 1]
    pub r_tox: f64,
    /// Normalized degradation shortfall risk r_deg in [0, 1]
    pub r_deg: f64,
    /// Lyapunov-style residual V = w1 * r_tox + w2 * r_deg
    pub v_residual: f64,
    /// Hard pass/fail flag for corridor compliance
    pub corridor_safe: bool,
}

pub fn simulate_decomposition(params: DecompositionParams) -> DecompositionState {
    let mut c_t = params.c0;
    let mut day_first_safe: Option<u32> = None;

    for day in 0..=params.t_max_days {
        let t = day as f64;
        c_t = params.c0 * E.powf(-params.k * t);

        if c_t <= params.tox_limit && day_first_safe.is_none() {
            day_first_safe = Some(day);
        }
    }

    let c_final = c_t;
    let frac_degraded_final = if params.c0 > 0.0 {
        ((params.c0 - c_final) / params.c0).clamp(0.0, 1.0)
    } else {
        1.0
    };

    // Toxicity risk: 0 if safe by corridor_day, 1 if never safe within horizon,
    // interpolate if safe after corridor_day but before t_max_days.
    let r_tox = match day_first_safe {
        Some(day) if day <= params.corridor_day => 0.0,
        Some(day) if day > params.t_max_days => 1.0,
        Some(day) => {
            let num = (day.saturating_sub(params.corridor_day)) as f64;
            let den = (params.t_max_days.saturating_sub(params.corridor_day).max(1)) as f64;
            (num / den).clamp(0.0, 1.0)
        }
        None => 1.0,
    };

    // Degradation risk: 0 if target achieved, increases linearly with shortfall.
    let r_deg = if params.target_deg_fraction <= 0.0 {
        0.0
    } else {
        let shortfall = (params.target_deg_fraction - frac_degraded_final).max(0.0);
        (shortfall / params.target_deg_fraction).clamp(0.0, 1.0)
    };

    // Weights can be tuned per ALN corridor shard; here toxicity is emphasized.
    let w_tox = 0.7_f64;
    let w_deg = 0.3_f64;
    let v_residual = w_tox * r_tox + w_deg * r_deg;

    // Corridor-safe if: safe by corridor_day AND target degradation met by t_max_days.
    let corridor_safe = (r_tox == 0.0) && (r_deg == 0.0);

    DecompositionState {
        c_final,
        frac_degraded_final,
        day_first_safe,
        r_tox,
        r_deg,
        v_residual,
        corridor_safe,
    }
}

fn main() {
    // Example: biodegradable tray leachate corridor aligned with ISO 14851-style targets.
    let params = DecompositionParams {
        c0: 100.0,          // mg/kg or mg/L
        k: 0.05,            // 1/day
        t_max_days: 180,
        tox_limit: 0.001,   // mg/kg or mg/L
        corridor_day: 90,   // must be below tox_limit by day 90
        target_deg_fraction: 0.90, // at least 90% degraded by day 180
    };

    let state = simulate_decomposition(params);

    println!("Final concentration: {:.6} units", state.c_final);
    println!("Final degraded fraction: {:.3}", state.frac_degraded_final);
    println!(
        "Day first safe (<= tox_limit): {}",
        state.day_first_safe
            .map(|d| d.to_string())
            .unwrap_or_else(|| "never within horizon".to_string())
    );
    println!("r_tox: {:.3}, r_deg: {:.3}, V: {:.3}", state.r_tox, state.r_deg, state.v_residual);
    println!("Corridor-safe: {}", state.corridor_safe);
}

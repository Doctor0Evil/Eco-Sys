use std::f64::consts::E;

#[derive(Clone, Copy, Debug)]
pub struct GeometricalParams {
    // Human thermal corridor
    pub wbgt_current: f64,   // current WBGT °C
    pub wbgt_safe: f64,      // safe band °C
    pub wbgt_hard: f64,      // hard limit °C

    // Aquifer plume corridor at compliance depth
    pub delta_t: f64,        // plume ΔT °C
    pub delta_gold: f64,     // eco-optimal plume limit °C
    pub delta_hard: f64,     // hard plume limit °C

    // Biodegradable substrate / toxicity surrogate
    pub c0: f64,             // initial concentration (normalized units)
    pub tox_limit: f64,      // toxicity corridor limit (same units as c0)
    pub k_decay: f64,        // decay constant 1/day
    pub corridor_day: u32,   // day by which c(t) must fall below tox_limit
    pub t_max_days: u32,     // simulation horizon days
}

#[derive(Clone, Copy, Debug)]
pub struct GeometricalState {
    pub r_wb: f64,           // 0–1 WBGT risk
    pub r_plume: f64,        // 0–1 plume thermal risk
    pub r_tox: f64,          // 0–1 toxicity / degradation risk
    pub v_residual: f64,     // Lyapunov residual
    pub day_first_safe: Option<u32>,
    pub corridor_safe: bool, // hard pass/fail for all three corridors
}

pub fn compute_risk_coordinates(params: GeometricalParams) -> GeometricalState {
    // WBGT risk
    let r_wb = if params.wbgt_current <= params.wbgt_safe {
        0.0
    } else if params.wbgt_current < params.wbgt_hard {
        (params.wbgt_current - params.wbgt_safe)
            / (params.wbgt_hard - params.wbgt_safe)
    } else {
        1.0
    };

    // Plume thermal risk (absolute ΔT)
    let dt_abs = params.delta_t.abs();
    let r_plume = if dt_abs <= params.delta_gold {
        0.0
    } else if dt_abs < params.delta_hard {
        (dt_abs - params.delta_gold) / (params.delta_hard - params.delta_gold)
    } else {
        1.0
    };

    // Toxicity / degradation corridor
    let mut c_t = params.c0;
    let mut day_first_safe: Option<u32> = None;

    for day in 0..=params.t_max_days {
        let t = day as f64;
        c_t = params.c0 * E.powf(-params.k_decay * t);

        if c_t <= params.tox_limit && day_first_safe.is_none() {
            day_first_safe = Some(day);
        }
    }

    // Normalize toxicity risk:
    //  - 0 if safe by corridor_day
    //  - 1 if never safe within horizon
    //  - interpolate if safe after corridor_day but before t_max_days
    let r_tox = match day_first_safe {
        Some(day) if day <= params.corridor_day => 0.0,
        Some(day) if day >= params.t_max_days => 1.0,
        Some(day) => {
            let num = day.saturating_sub(params.corridor_day) as f64;
            let den = (params.t_max_days.saturating_sub(params.corridor_day)).max(1) as f64;
            (num / den).clamp(0.0, 1.0)
        }
        None => 1.0,
    };

    // Residual with explicit weights (can be shard-parameterized)
    let w_wb = 0.4_f64;
    let w_plume = 0.3_f64;
    let w_tox = 0.3_f64;

    let v_residual = w_wb * r_wb + w_plume * r_plume + w_tox * r_tox;

    // Hard corridor pass: all risks must be zero (gold corridor)
    let corridor_safe = (r_wb == 0.0) && (r_plume == 0.0) && (r_tox == 0.0);

    GeometricalState {
        r_wb,
        r_plume,
        r_tox,
        v_residual,
        day_first_safe,
        corridor_safe,
    }
}

fn main() {
    let params = GeometricalParams {
        wbgt_current: 32.0,
        wbgt_safe: 31.0,
        wbgt_hard: 35.0,
        delta_t: 0.4,
        delta_gold: 0.3,
        delta_hard: 0.5,
        c0: 1.0,          // normalized
        tox_limit: 0.001, // normalized corridor limit
        k_decay: 0.05,
        corridor_day: 90,
        t_max_days: 180,
    };

    let state = compute_risk_coordinates(params);

    println!(
        "WBGT Risk: {:.3}, Plume Risk: {:.3}, Tox Risk: {:.3}, V: {:.3}, First safe day: {:?}, Corridor safe: {}",
        state.r_wb, state.r_plume, state.r_tox, state.v_residual, state.day_first_safe, state.corridor_safe
    );
}

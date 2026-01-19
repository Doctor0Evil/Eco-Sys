use std::f64::consts::E;
#[derive(Clone, Copy)]
struct GeometricalParams {
wbgt_current: f64,  // current WBGT °C
wbgt_safe: f64,     // safe threshold °C
wbgt_hard: f64,     // hard limit °C
delta_t: f64,       // aquifer plume ∆T °C
delta_gold: f64,    // gold plume limit °C
delta_hard: f64,    // hard plume limit °C
volume_zone: f64,   // geometrical volume m³
k_decay: f64,       // decay constant /day for substrates
t_max: f64,         // max analysis time days
}
fn compute_risk_coordinates(params: GeometricalParams) -> (f64, f64, f64, bool) {
let r_wb = if params.wbgt_current <= params.wbgt_safe {
0.0
} else if params.wbgt_current < params.wbgt_hard {
(params.wbgt_current - params.wbgt_safe) / (params.wbgt_hard - params.wbgt_safe)
} else {
1.0
};
let r_plume = if params.delta_t.abs() <= params.delta_gold {
0.0
} else if params.delta_t.abs() < params.delta_hard {
(params.delta_t.abs() - params.delta_gold) / (params.delta_hard - params.delta_gold)
} else {
1.0
};
let mut c_t = params.volume_zone;  // initial "concentration" proxy
let mut t = 0.0;
let mut safe = true;
while t < params.t_max {
c_t = params.volume_zone * E.powf(-params.k_decay * t);
if c_t > 0.001 * params.volume_zone {  // toxicity proxy threshold
safe = false;
break;
}
t += 1.0;
}
let v_residual = 0.5 * r_wb + 0.3 * r_plume + 0.2 * (c_t / params.volume_zone);
(r_wb, r_plume, v_residual, safe)
}
fn main() {
let params = GeometricalParams {
wbgt_current: 32.0,
wbgt_safe: 31.0,
wbgt_hard: 35.0,
delta_t: 0.4,
delta_gold: 0.3,
delta_hard: 0.5,
volume_zone: 1000.0,
k_decay: 0.05,
t_max: 180.0,
};
let (r_wb, r_plume, v_residual, is_safe) = compute_risk_coordinates(params);
println!("WBGT Risk: {}, Plume Risk: {}, Residual V: {}, Safe: {}", r_wb, r_plume, v_residual, is_safe);
}

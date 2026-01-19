use std::f64::consts::E;
#[derive(Clone, Copy)]
struct DecompositionParams {
c0: f64,  // initial concentration mg/kg
k: f64,   // decay constant /day
t_max: f64,  // max time days
tox_threshold: f64,  // toxicity limit mg/kg
}
fn simulate_decomposition(params: DecompositionParams) -> (f64, bool) {
let mut c_t = params.c0;
let mut t = 0.0;
let mut safe = true;
while t < params.t_max {
c_t = params.c0 * E.powf(-params.k * t);
if c_t > params.tox_threshold {
safe = false;
break;
}
t += 1.0;
}
(c_t, safe)
}
fn main() {
let params = DecompositionParams {
c0: 100.0,
k: 0.05,
t_max: 180.0,
tox_threshold: 0.001,
};
let (final_c, is_safe) = simulate_decomposition(params);
println!("Final concentration: {} mg/kg, Safe: {}", final_c, is_safe);
}

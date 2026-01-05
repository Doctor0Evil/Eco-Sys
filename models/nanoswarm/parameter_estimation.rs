use std::f64::INFINITY;

// Basic time-stamped measurement at a grid cell for one pollutant
#[derive(Clone, Debug)]
pub struct Measurement {
    pub time: f64,        // seconds
    pub cell_id: usize,   // spatial cell index j
    pub pollutant_id: usize, // index i
    pub value: f64,       // measured concentration [mol/m^3]
}

// Discrete 1D grid model for simplicity; can be generalized to 2D/3D
#[derive(Clone)]
pub struct Grid1D {
    pub n_cells: usize,
    pub dx: f64,          // cell size [m]
}

// Parameters being estimated for each pollutant i
#[derive(Clone)]
pub struct NanoswarmParams {
    pub d_pollutant: Vec<f64>, // D_i, diffusivity [m^2/s]
    pub k_removal: Vec<f64>,   // k_i, nanoswarm removal rate [m^3/(agent·s)]
    pub lambda_loss: f64,      // nanoswarm loss rate [1/s]
}

// Static configuration and safety bounds
#[derive(Clone)]
pub struct CalibrationConfig {
    pub n_pollutants: usize,
    pub dt: f64,                 // numerical time step [s]
    pub max_steps: usize,        // max simulation steps in each forward run
    pub d_min: f64,
    pub d_max: f64,
    pub k_min: f64,
    pub k_max: f64,
    pub lambda_min: f64,
    pub lambda_max: f64,
    pub safe_thresholds: Vec<f64>,   // safe max concentration per pollutant
    pub safety_weight: f64,          // penalty weight for exceeding threshold
    pub learning_rate: f64,          // gradient step size
}

// State of pollutant fields and swarm density at grid cells
#[derive(Clone)]
pub struct NanoswarmState1D {
    pub grid: Grid1D,
    pub pollutants: Vec<Vec<f64>>, // pollutants[i][j] = c_{i,j}
    pub rho: Vec<f64>,             // swarm density per cell [agents/m^3]
    pub adv_velocity: f64,         // 1D advection velocity u [m/s]
}

// Helper: clamp value to [lo, hi]
fn clamp(x: f64, lo: f64, hi: f64) -> f64 {
    if x < lo { lo } else if x > hi { hi } else { x }
}

// One explicit finite-difference update for pollutants and swarm density
fn step_state(
    state: &mut NanoswarmState1D,
    params: &NanoswarmParams,
    cfg: &CalibrationConfig,
) {
    let n = state.grid.n_cells;
    let dx = state.grid.dx;
    let dt = cfg.dt;
    let u = state.adv_velocity;

    // Copy old values
    let old_pollutants = state.pollutants.clone();
    let old_rho = state.rho.clone();

    // Update pollutant concentrations
    for i in 0..cfg.n_pollutants {
        let d = params.d_pollutant[i];
        let k = params.k_removal[i];

        for j in 0..n {
            let c = old_pollutants[i][j];
            let rho = old_rho[j];

            // Upwind advection (1D)
            let adv = if u >= 0.0 {
                let c_left = if j > 0 { old_pollutants[i][j - 1] } else { c };
                -u * (c - c_left) / dx
            } else {
                let c_right = if j + 1 < n { old_pollutants[i][j + 1] } else { c };
                -u * (c_right - c) / dx
            };

            // Diffusion (central)
            let c_left = if j > 0 { old_pollutants[i][j - 1] } else { c };
            let c_right = if j + 1 < n { old_pollutants[i][j + 1] } else { c };
            let diff = d * (c_left - 2.0 * c + c_right) / (dx * dx);

            // Reaction (removal by nanoswarm)
            let reaction = -k * rho * c;

            let c_new = c + dt * (adv + diff + reaction);

            state.pollutants[i][j] = if c_new < 0.0 { 0.0 } else { c_new };
        }
    }

    // Simple exponential loss on swarm density (no advection/diffusion here)
    for j in 0..n {
        let rho_old = old_rho[j];
        let decay = -params.lambda_loss * rho_old;
        let rho_new = rho_old + dt * decay;
        state.rho[j] = if rho_new < 0.0 { 0.0 } else { rho_new };
    }
}

// Compute misfit between simulated concentrations and measurements
// plus safety penalties for exceeding ecological thresholds
fn compute_loss(
    simulated: &NanoswarmState1D,
    measurements: &[Measurement],
    cfg: &CalibrationConfig,
) -> f64 {
    let mut loss = 0.0;

    for m in measurements {
        if m.pollutant_id >= cfg.n_pollutants { continue; }
        if m.cell_id >= simulated.grid.n_cells { continue; }

        // For simplicity, ignore time stamps and use final state as proxy;
        // extension: store trajectory and interpolate by time.
        let c_model = simulated.pollutants[m.pollutant_id][m.cell_id];
        let diff = c_model - m.value;
        loss += diff * diff;
    }

    // Ecological safety: penalize concentrations above thresholds
    for i in 0..cfg.n_pollutants {
        let threshold = cfg.safe_thresholds[i];
        for c in &simulated.pollutants[i] {
            if *c > threshold {
                let excess = c - threshold;
                loss += cfg.safety_weight * excess * excess;
            }
        }
    }

    loss
}

// Numerical gradient approximation by finite differences
fn estimate_gradient(
    base_state: &NanoswarmState1D,
    params: &NanoswarmParams,
    cfg: &CalibrationConfig,
    measurements: &[Measurement],
) -> (Vec<f64>, Vec<f64>, f64) {
    let eps = 1e-5;

    // Baseline loss
    let mut state0 = base_state.clone();
    for _ in 0..cfg.max_steps {
        step_state(&mut state0, params, cfg);
    }
    let base_loss = compute_loss(&state0, measurements, cfg);

    // Gradients for D_i
    let mut grad_d = vec![0.0; cfg.n_pollutants];
    for i in 0..cfg.n_pollutants {
        let mut p_eps = params.clone();
        p_eps.d_pollutant[i] += eps;

        let mut s_eps = base_state.clone();
        for _ in 0..cfg.max_steps {
            step_state(&mut s_eps, &p_eps, cfg);
        }
        let loss_eps = compute_loss(&s_eps, measurements, cfg);
        grad_d[i] = (loss_eps - base_loss) / eps;
    }

    // Gradients for k_i
    let mut grad_k = vec![0.0; cfg.n_pollutants];
    for i in 0..cfg.n_pollutants {
        let mut p_eps = params.clone();
        p_eps.k_removal[i] += eps;

        let mut s_eps = base_state.clone();
        for _ in 0..cfg.max_steps {
            step_state(&mut s_eps, &p_eps, cfg);
        }
        let loss_eps = compute_loss(&s_eps, measurements, cfg);
        grad_k[i] = (loss_eps - base_loss) / eps;
    }

    // Gradient for lambda_loss
    let mut p_eps = params.clone();
    p_eps.lambda_loss += eps;
    let mut s_eps = base_state.clone();
    for _ in 0..cfg.max_steps {
        step_state(&mut s_eps, &p_eps, cfg);
    }
    let loss_eps = compute_loss(&s_eps, measurements, cfg);
    let grad_lambda = (loss_eps - base_loss) / eps;

    (grad_d, grad_k, grad_lambda)
}

// Single calibration iteration: update parameters given measurements
pub fn calibration_step(
    base_state: &NanoswarmState1D,
    params: &mut NanoswarmParams,
    cfg: &CalibrationConfig,
    measurements: &[Measurement],
) -> f64 {
    let (grad_d, grad_k, grad_lambda) =
        estimate_gradient(base_state, params, cfg, measurements);

    // Gradient-descent update with clamping to safety/physical bounds
    for i in 0..cfg.n_pollutants {
        params.d_pollutant[i] =
            clamp(params.d_pollutant[i] - cfg.learning_rate * grad_d[i],
                  cfg.d_min, cfg.d_max);
        params.k_removal[i] =
            clamp(params.k_removal[i] - cfg.learning_rate * grad_k[i],
                  cfg.k_min, cfg.k_max);
    }
    params.lambda_loss =
        clamp(params.lambda_loss - cfg.learning_rate * grad_lambda,
              cfg.lambda_min, cfg.lambda_max);

    // Compute loss after update
    let mut state_updated = base_state.clone();
    for _ in 0..cfg.max_steps {
        step_state(&mut state_updated, params, cfg);
    }
    compute_loss(&state_updated, measurements, cfg)
}

// High-level loop: continuously integrate new data and re-calibrate
pub fn online_calibration_loop(
    mut base_state: NanoswarmState1D,
    mut params: NanoswarmParams,
    cfg: CalibrationConfig,
    data_stream: impl Iterator<Item = Vec<Measurement>>,
    max_iters: usize,
) -> (NanoswarmState1D, NanoswarmParams) {
    let mut best_params = params.clone();
    let mut best_loss = INFINITY;

    for (iter, batch) in data_stream.take(max_iters).enumerate() {
        let loss = calibration_step(&base_state, &mut params, &cfg, &batch);

        // Optionally update base_state using current best parameters
        for _ in 0..cfg.max_steps {
            step_state(&mut base_state, &params, &cfg);
        }

        if loss < best_loss {
            best_loss = loss;
            best_params = params.clone();
        }

        println!(
            "Iteration {}: loss = {:.6}, lambda_loss = {:.3}",
            iter, loss, params.lambda_loss
        );
    }

    (base_state, best_params)
}

// Example: main wiring (to be extended to real data sources and orchestration policies)
fn main() {
    let grid = Grid1D { n_cells: 50, dx: 1.0 };
    let n_pollutants = 2;

    // Initial state: some pollutant plume and uniform swarm density
    let initial_pollutants = vec![
        vec![1.0; grid.n_cells], // pollutant 0
        vec![0.5; grid.n_cells], // pollutant 1
    ];
    let initial_rho = vec![1e6; grid.n_cells]; // agents/m^3

    let base_state = NanoswarmState1D {
        grid,
        pollutants: initial_pollutants,
        rho: initial_rho,
        adv_velocity: 0.01, // m/s
    };

    let params = NanoswarmParams {
        d_pollutant: vec![1e-3; n_pollutants],
        k_removal: vec![1e-7; n_pollutants],
        lambda_loss: 1e-5,
    };

    let cfg = CalibrationConfig {
        n_pollutants,
        dt: 1.0,
        max_steps: 50,
        d_min: 1e-6,
        d_max: 1e-1,
        k_min: 1e-9,
        k_max: 1e-4,
        lambda_min: 0.0,
        lambda_max: 1e-3,
        safe_thresholds: vec![0.1, 0.05],
        safety_weight: 10.0,
        learning_rate: 1e-2,
    };

    // Placeholder data stream: in production, this would pull from
    // nanoswarm sensor logs / telemetry buffers
    let dummy_stream = (0..100).map(|step| {
        let t = step as f64;
        let mut batch = Vec::new();
        for cell in [10usize, 25, 40] {
            for pid in 0..n_pollutants {
                batch.push(Measurement {
                    time: t,
                    cell_id: cell,
                    pollutant_id: pid,
                    // target: drive concentrations toward zero
                    value: 0.0,
                });
            }
        }
        batch
    });

    let (_final_state, final_params) =
        online_calibration_loop(base_state, params, cfg, dummy_stream, 50);

    println!("Calibrated parameters:");
    println!("  d_pollutant = {:?}", final_params.d_pollutant);
    println!("  k_removal   = {:?}", final_params.k_removal);
    println!("  lambda_loss = {:?}", final_params.lambda_loss);
}

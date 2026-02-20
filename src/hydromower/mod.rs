// Hex-stamp placeholder (HydroMower2026v1)
// 0xhydro_mower_2026_v1_eca17f9b42de901277aa55cc33991188
#![forbid(unsafe_code)]

use crate::econet_core::{RiskCoord, Residual, CorridorDecision, enforce_safe_step};
use crate::region::RegionConfig;

/// Raw physical test record for one mower configuration.
#[derive(Clone, Debug)]
pub struct HydroMowerRun {
    pub nodeid: String,
    pub region: String,
    pub lat: f64,
    pub lon: f64,

    pub cutarea_m2_per_cycle: f64,
    pub nozzle_pressure_bar: f64,
    pub flow_l_per_min: f64,
    pub runtime_min: f64,

    pub wateruse_l_per_m2: f64,
    pub energy_equiv_kwh: f64,
    pub noise_dba: f64,
    pub spray_drift_m: f64,
}

/// Normalized eco-scores for a mower run.
#[derive(Clone, Debug)]
pub struct HydroMowerScore {
    pub rcutquality: f64,
    pub rwateruse: f64,
    pub rnoise: f64,
    pub rspraydrift: f64,
    pub rsafety: f64,

    pub ecoimpact_01: f64,
    pub riskofharm_01: f64,
    pub knowledgefactor_01: f64,
    pub violation_residual_Vt: f64,
}

/// Shard row ready to write to
/// qpudatashards/particles/HydroMowerPhoenix2026v1.csv.
#[derive(Clone, Debug)]
pub struct HydroMowerShardRow {
    pub nodeid: String,
    pub region: String,
    pub lat: f64,
    pub lon: f64,

    pub cutarea_m2_per_cycle: f64,
    pub nozzle_pressure_bar: f64,
    pub flow_l_per_min: f64,
    pub runtime_min: f64,

    pub wateruse_l_per_m2: f64,
    pub energy_equiv_kwh: f64,
    pub noise_dba: f64,
    pub spray_drift_m: f64,

    pub rcutquality_01: f64,
    pub rwateruse_01: f64,
    pub rnoise_01: f64,
    pub rspraydrift_01: f64,
    pub rsafety_01: f64,

    pub ecoimpact_01: f64,
    pub riskofharm_01: f64,
    pub knowledgefactor_01: f64,
    pub violation_residual_Vt: f64,

    pub hexstamp: String,
    pub notes: String,
}

/// Map raw measurements to normalized risk coordinates.
/// Numbers here are placeholders; you will calibrate using Phoenix baselines.
fn score_hydromower_run(run: &HydroMowerRun, region: &dyn RegionConfig) -> HydroMowerScore {
    // Example corridor-normalization helpers (linear ramps).
    fn ramp(x: f64, safe: f64, hard: f64) -> f64 {
        if x <= safe {
            0.0
        } else if x >= hard {
            1.0
        } else {
            (x - safe) / (hard - safe)
        }
    }

    // rwateruse: compare to "baseline irrigation + mowing" corridor.
    // You will replace constants with RegionConfig methods later.
    let rwateruse = ramp(run.wateruse_l_per_m2, 2.0, 8.0);

    // rnoise: quiet ≤ 70 dBA, hard band at 90 dBA.
    let rnoise = ramp(run.noise_dba, 70.0, 90.0);

    // rspraydrift: drift radius ≤ 0.5 m safe, ≥ 2 m hard.
    let rspraydrift = ramp(run.spray_drift_m, 0.5, 2.0);

    // rcutquality: you will compute this from field cut-height variance;
    // placeholder uses wateruse as proxy for now.
    let rcutquality = rwateruse.min(1.0);

    // rsafety: a composite of pressure, exposed jets, and containment.
    let rsafety = if run.nozzle_pressure_bar <= 6.0 { 0.2 } else { 0.8 };

    // Aggregate into Residual for Lyapunov-style safety.
    let rx = vec![
        RiskCoord {
            varid: "rcutquality".to_string(),
            value: rcutquality,
            safe: 0.3,
            gold: 0.6,
            hard: 1.0,
            weight: 0.25,
            lyapchannel: 30,
        },
        RiskCoord {
            varid: "rwateruse".to_string(),
            value: rwateruse,
            safe: 0.3,
            gold: 0.6,
            hard: 1.0,
            weight: 0.25,
            lyapchannel: 31,
        },
        RiskCoord {
            varid: "rnoise".to_string(),
            value: rnoise,
            safe: 0.3,
            gold: 0.7,
            hard: 1.0,
            weight: 0.20,
            lyapchannel: 32,
        },
        RiskCoord {
            varid: "rspraydrift".to_string(),
            value: rspraydrift,
            safe: 0.2,
            gold: 0.6,
            hard: 1.0,
            weight: 0.15,
            lyapchannel: 33,
        },
        RiskCoord {
            varid: "rsafety".to_string(),
            value: rsafety,
            safe: 0.1,
            gold: 0.5,
            hard: 1.0,
            weight: 0.15,
            lyapchannel: 34,
        },
    ];

    let mut residual = Residual {
        vt: 0.0,
        weights: rx.iter().map(|r| r.weight).collect(),
        rx,
    };
    residual.recompute();

    // Ecoimpact: simple placeholder comparing to gas mower fuel use.
    // Later you can tie this to CEIM LCA for gas vs hydro vs electric.
    let ecoimpact_01 = (1.0 - rwateruse).max(0.0);

    // Risk-of-harm as weighted sum of rx (already folded into vt).
    let riskofharm_01 = residual.vt.min(1.0);

    HydroMowerScore {
        rcutquality,
        rwateruse,
        rnoise,
        rspraydrift,
        rsafety,
        ecoimpact_01,
        riskofharm_01,
        knowledgefactor_01: 0.93,
        violation_residual_Vt: residual.vt,
    }
}

/// Convert run + score into a shard row and enforce ecosafety invariants.
/// If any hard corridor is hit or Vt increases vs previous, derate/stop upstream.
pub fn hydromower_to_shard_row(
    prev_residual: &Residual,
    run: &HydroMowerRun,
    region: &dyn RegionConfig,
    hexstamp: String,
    notes: String,
) -> (CorridorDecision, HydroMowerShardRow, Residual) {
    let score = score_hydromower_run(run, region);

    // Rebuild residual to feed into enforce_safe_step.
    let rx = vec![
        RiskCoord {
            varid: "rcutquality".to_string(),
            value: score.rcutquality,
            safe: 0.3,
            gold: 0.6,
            hard: 1.0,
            weight: 0.25,
            lyapchannel: 30,
        },
        RiskCoord {
            varid: "rwateruse".to_string(),
            value: score.rwateruse,
            safe: 0.3,
            gold: 0.6,
            hard: 1.0,
            weight: 0.25,
            lyapchannel: 31,
        },
        RiskCoord {
            varid: "rnoise".to_string(),
            value: score.rnoise,
            safe: 0.3,
            gold: 0.7,
            hard: 1.0,
            weight: 0.20,
            lyapchannel: 32,
        },
        RiskCoord {
            varid: "rspraydrift".to_string(),
            value: score.rspraydrift,
            safe: 0.2,
            gold: 0.6,
            hard: 1.0,
            weight: 0.15,
            lyapchannel: 33,
        },
        RiskCoord {
            varid: "rsafety".to_string(),
            value: score.rsafety,
            safe: 0.1,
            gold: 0.5,
            hard: 1.0,
            weight: 0.15,
            lyapchannel: 34,
        },
    ];
    let mut next_residual = Residual {
        vt: 0.0,
        weights: rx.iter().map(|r| r.weight).collect(),
        rx,
    };
    next_residual.recompute();

    let decision = enforce_safe_step(prev_residual.clone(), next_residual.clone());

    let row = HydroMowerShardRow {
        nodeid: run.nodeid.clone(),
        region: run.region.clone(),
        lat: run.lat,
        lon: run.lon,

        cutarea_m2_per_cycle: run.cutarea_m2_per_cycle,
        nozzle_pressure_bar: run.nozzle_pressure_bar,
        flow_l_per_min: run.flow_l_per_min,
        runtime_min: run.runtime_min,

        wateruse_l_per_m2: run.wateruse_l_per_m2,
        energy_equiv_kwh: run.energy_equiv_kwh,
        noise_dba: run.noise_dba,
        spray_drift_m: run.spray_drift_m,

        rcutquality_01: score.rcutquality,
        rwateruse_01: score.rwateruse,
        rnoise_01: score.rnoise,
        rspraydrift_01: score.rspraydrift,
        rsafety_01: score.rsafety,

        ecoimpact_01: score.ecoimpact_01,
        riskofharm_01: score.riskofharm_01,
        knowledgefactor_01: score.knowledgefactor_01,
        violation_residual_Vt: next_residual.vt,

        hexstamp,
        notes,
    };

    (decision, row, next_residual)
}

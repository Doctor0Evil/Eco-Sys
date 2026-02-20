// Hex-stamp placeholder (LawnBiofuelPhoenix2026v1)
// 0xlawn_biofuel_2026_v1_eca17f9b42de901277aa55cc33991188
#![forbid(unsafe_code)]

use crate::econet_core::{RiskCoord, Residual, CorridorDecision, enforce_safe_step};
use crate::region::RegionConfig;

#[derive(Clone, Debug)]
pub enum ConversionRoute {
    Combustion,
    Digestion,
    Pyrolysis,
}

#[derive(Clone, Debug)]
pub struct LawnFeedstockSample {
    pub feedstock_id: String,
    pub region: String,
    pub lat: f64,
    pub lon: f64,

    pub route: ConversionRoute,
    pub moisture_frac: f64,
    pub ash_frac: f64,
    pub n_percent_dw: f64,

    pub t90_days: f64,
    pub contaminant_rtox_01: f64,
    pub microplastics_r_01: f64,
    pub heavy_metals_r_01: f64,

    pub bioenergy_kwh_per_kg: f64,
    pub co2eq_kg_per_kg: f64,
    pub transport_km: f64,
    pub rtrans_01: f64,
    pub rmat_01: f64,
    pub rman_01: f64,
}

#[derive(Clone, Debug)]
pub struct LawnBiofuelScore {
    pub ecoimpact_01: f64,
    pub riskofharm_01: f64,
    pub knowledgefactor_01: f64,
    pub violation_residual_Vt: f64,
}

#[derive(Clone, Debug)]
pub struct LawnBiofuelShardRow {
    pub feedstock_id: String,
    pub region: String,
    pub lat: f64,
    pub lon: f64,

    pub route: String,
    pub moisture_frac: f64,
    pub ash_frac: f64,
    pub n_percent_dw: f64,
    pub contaminant_rtox_01: f64,
    pub microplastics_r_01: f64,
    pub heavy_metals_r_01: f64,

    pub t90_days: f64,
    pub bioenergy_kwh_per_kg: f64,
    pub co2eq_kg_per_kg: f64,
    pub transport_km: f64,
    pub rtrans_01: f64,
    pub rmat_01: f64,
    pub rman_01: f64,

    pub ecoimpact_01: f64,
    pub riskofharm_01: f64,
    pub knowledgefactor_01: f64,
    pub violation_residual_Vt: f64,

    pub hexstamp: String,
    pub notes: String,
}

fn score_lawn_biofuel(
    s: &LawnFeedstockSample,
    region: &dyn RegionConfig,
) -> (LawnBiofuelScore, Residual) {
    // Biodegradation corridor (t90 target and hard from RegionConfig).
    let rt90 = if s.t90_days <= region.t90target_days() {
        0.0
    } else if s.t90_days >= region.t90hardlimit_days() {
        1.0
    } else {
        (s.t90_days - region.t90target_days())
            / (region.t90hardlimit_days() - region.t90target_days())
    };

    // Reuse rtox corridor bands from RegionConfig.
    let rtox = s.contaminant_rtox_01.max(0.0).min(1.0);
    let rmicro = s.microplastics_r_01.max(0.0).min(1.0);
    let rmetals = s.heavy_metals_r_01.max(0.0).min(1.0);
    let rtrans = s.rtrans_01.max(0.0).min(1.0);
    let rmat = s.rmat_01.max(0.0).min(1.0);
    let rman = s.rman_01.max(0.0).min(1.0);

    let rx = vec![
        RiskCoord {
            varid: "rt90".to_string(),
            value: rt90,
            safe: 0.3,
            gold: 0.6,
            hard: 1.0,
            weight: 0.20,
            lyapchannel: 40,
        },
        RiskCoord {
            varid: "rtox".to_string(),
            value: rtox,
            safe: region.rtox_safe(),
            gold: region.rtox_gold(),
            hard: region.rtox_hard(),
            weight: 0.25,
            lyapchannel: 41,
        },
        RiskCoord {
            varid: "rmicro".to_string(),
            value: rmicro,
            safe: 0.1,
            gold: 0.4,
            hard: 1.0,
            weight: 0.15,
            lyapchannel: 42,
        },
        RiskCoord {
            varid: "rmetals".to_string(),
            value: rmetals,
            safe: 0.1,
            gold: 0.4,
            hard: 1.0,
            weight: 0.10,
            lyapchannel: 43,
        },
        RiskCoord {
            varid: "rtrans".to_string(),
            value: rtrans,
            safe: 0.3,
            gold: 0.7,
            hard: 1.0,
            weight: 0.10,
            lyapchannel: 44,
        },
        RiskCoord {
            varid: "rmat".to_string(),
            value: rmat,
            safe: 0.3,
            gold: 0.7,
            hard: 1.0,
            weight: 0.10,
            lyapchannel: 45,
        },
        RiskCoord {
            varid: "rman".to_string(),
            value: rman,
            safe: 0.3,
            gold: 0.7,
            hard: 1.0,
            weight: 0.10,
            lyapchannel: 46,
        },
    ];

    let mut residual = Residual {
        vt: 0.0,
        weights: rx.iter().map(|r| r.weight).collect(),
        rx,
    };
    residual.recompute();

    // Ecoimpact: high when we get good energy, low CO2eq, and low transport.
    // A simple, normalized placeholder:
    let e_energy = (s.bioenergy_kwh_per_kg / 3.0).min(1.0); // assume 3 kWh/kg ~ high
    let e_carbon = (1.0 - (s.co2eq_kg_per_kg / 1.5)).max(0.0).min(1.0);
    let e_transport = 1.0 - rtrans;

    let ecoimpact_raw = 0.4 * e_energy + 0.4 * e_carbon + 0.2 * e_transport;

    // Hard gates: if any of rt90, rtox, rmicro, rmetals hit 1.0, collapse E to 0.
    let hard_gate = rt90 >= 1.0 || rtox >= 1.0 || rmicro >= 1.0 || rmetals >= 1.0;
    let ecoimpact_final = if hard_gate { 0.0 } else { ecoimpact_raw };

    let score = LawnBiofuelScore {
        ecoimpact_01: ecoimpact_final,
        riskofharm_01: residual.vt.min(1.0),
        knowledgefactor_01: 0.93,
        violation_residual_Vt: residual.vt,
    };

    (score, residual)
}

pub fn lawn_biofuel_to_shard_row(
    prev_residual: &Residual,
    sample: &LawnFeedstockSample,
    region: &dyn RegionConfig,
    hexstamp: String,
    notes: String,
) -> (CorridorDecision, LawnBiofuelShardRow, Residual) {
    let (score, mut residual) = score_lawn_biofuel(sample, region);
    residual.recompute();

    let decision = enforce_safe_step(prev_residual.clone(), residual.clone());

    let route_str = match sample.route {
        ConversionRoute::Combustion => "COMBUSTION",
        ConversionRoute::Digestion => "DIGESTION",
        ConversionRoute::Pyrolysis => "PYROLYSIS",
    }
    .to_string();

    let row = LawnBiofuelShardRow {
        feedstock_id: sample.feedstock_id.clone(),
        region: sample.region.clone(),
        lat: sample.lat,
        lon: sample.lon,

        route: route_str,
        moisture_frac: sample.moisture_frac,
        ash_frac: sample.ash_frac,
        n_percent_dw: sample.n_percent_dw,
        contaminant_rtox_01: sample.contaminant_rtox_01,
        microplastics_r_01: sample.microplastics_r_01,
        heavy_metals_r_01: sample.heavy_metals_r_01,

        t90_days: sample.t90_days,
        bioenergy_kwh_per_kg: sample.bioenergy_kwh_per_kg,
        co2eq_kg_per_kg: sample.co2eq_kg_per_kg,
        transport_km: sample.transport_km,
        rtrans_01: sample.rtrans_01,
        rmat_01: sample.rmat_01,
        rman_01: sample.rman_01,

        ecoimpact_01: score.ecoimpact_01,
        riskofharm_01: score.riskofharm_01,
        knowledgefactor_01: score.knowledgefactor_01,
        violation_residual_Vt: residual.vt,

        hexstamp,
        notes,
    };

    (decision, row, residual)
}

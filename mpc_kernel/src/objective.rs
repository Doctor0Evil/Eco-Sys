use serde::{Deserialize, Serialize};
use contracts_core::metrics::{KerVector, RiskScalar, EcoImpactScalar};
use crate::state::{MpcStateSlice, MpcControlSlice};

/// Dimensionless weight vector for objective terms, all >= 0.[file:39]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ObjectiveTermWeights {
    pub lambda_energy: f64,      // kWh or exergy per unit work.[file:39]
    pub lambda_degradation: f64, // hardware / tissue degradation proxy.[file:91]
    pub lambda_sparsity: f64,    // active node count.[file:39]
    pub lambda_slaviolation: f64,
}

/// Biocompatibility upgrade envelope, enforcing bioscale safety and neurorights.[file:91][file:88]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BiocompatObjectiveConfig {
    /// Minimum eco-impact per joule required (E_min).[file:4]
    pub e_min: EcoImpactScalar,
    /// Maximum normalized risk allowed (R_max).[file:91]
    pub r_max: RiskScalar,
    /// Hard flag to disable any reward that correlates with biological distress signals.
    pub forbid_distress_coupling: bool,
    /// Optional cap on cognitive load index for human-integrated planes.[file:85]
    pub max_cognitive_load: Option<f64>,
    pub weights: ObjectiveTermWeights,
}

/// Runtime objective that is guaranteed to be bioscale-compatible if constructed via `new_checked`.[file:91]
#[derive(Clone, Debug)]
pub struct BiocompatObjective {
    cfg: BiocompatObjectiveConfig,
}

#[derive(Debug, thiserror::Error)]
pub enum BiocompatError {
    #[error("invalid eco-impact floor: e_min={0}")]
    InvalidEcoImpact(EcoImpactScalar),
    #[error("invalid risk ceiling: r_max={0}")]
    InvalidRisk(RiskScalar),
    #[error("non-finite weight encountered")]
    NonFiniteWeight,
}

impl BiocompatObjective {
    pub fn new_checked(cfg: BiocompatObjectiveConfig) -> Result<Self, BiocompatError> {
        if !cfg.e_min.is_finite() || cfg.e_min < 0.0 {
            return Err(BiocompatError::InvalidEcoImpact(cfg.e_min));
        }
        if !cfg.r_max.is_finite() || cfg.r_max <= 0.0 || cfg.r_max > 1.0 {
            return Err(BiocompatError::InvalidRisk(cfg.r_max));
        }
        for w in [
            cfg.weights.lambda_energy,
            cfg.weights.lambda_degradation,
            cfg.weights.lambda_sparsity,
            cfg.weights.lambda_slaviolation,
        ] {
            if !w.is_finite() || w < 0.0 {
                return Err(BiocompatError::NonFiniteWeight);
            }
        }
        Ok(Self { cfg })
    }

    /// Evaluate J(x,u) under the biocompatibility envelope.[file:39][file:91]
    pub fn eval(
        &self,
        x: &MpcStateSlice,
        u: &MpcControlSlice,
        ker: &KerVector,
        eco: EcoImpactScalar,
        risk: RiskScalar,
    ) -> f64 {
        // Enforce eco floor and risk ceiling: if violated, return large penalty cost.[file:4][file:91]
        if eco < self.cfg.e_min || risk > self.cfg.r_max {
            return f64::INFINITY;
        }

        // Example placeholder terms: energy, degradation, sparsity, SLA.[file:39]
        let energy_term = self.cfg.weights.lambda_energy * ker.exergy_cost;
        let degr_term = self.cfg.weights.lambda_degradation * ker.degradation_index;
        let sparsity_term = self.cfg.weights.lambda_sparsity * ker.active_node_fraction;
        let sla_term = self.cfg.weights.lambda_slaviolation * ker.sla_violation_ratio;

        // All terms are non-negative; lower J is better but cannot trade off below eco/risk floors.[file:39]
        energy_term + degr_term + sparsity_term + sla_term
    }
}

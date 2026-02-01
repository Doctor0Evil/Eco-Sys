use serde::{Deserialize, Serialize};
use contracts_core::bioscale::{NeuroRightsSnapshot, BioIntegrationProfile};
use contracts_core::metrics::{RiskScalar, EcoImpactScalar};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BiocompatGuardConfig {
    /// Maximum allowed bioscale risk index for any organically-integrated component.[file:91]
    pub max_bio_risk: RiskScalar,
    /// Minimum eco-impact per joule for any human-coupled reward channel.[file:92]
    pub min_bio_eco: EcoImpactScalar,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BiocompatGuard {
    cfg: BiocompatGuardConfig,
}

#[derive(Debug, thiserror::Error)]
pub enum BiocompatViolation {
    #[error("bioscale risk index too high: R={0}")]
    RiskTooHigh(RiskScalar),
    #[error("bioscale eco-impact too low: E={0}")]
    EcoImpactTooLow(EcoImpactScalar),
    #[error("reward function couples to prohibited distress or coercion signals")]
    DistressCoupling,
}

impl BiocompatGuard {
    pub fn new(cfg: BiocompatGuardConfig) -> Self {
        Self { cfg }
    }

    /// Check that any organically-integrated interface remains within safety envelopes.[file:92][file:85]
    pub fn check_neurorights(
        &self,
        neuro: &NeuroRightsSnapshot,
        profile: &BioIntegrationProfile,
    ) -> Result<(), BiocompatViolation> {
        let r = neuro.normalized_risk;
        let e = neuro.eco_impact_index;

        if r > self.cfg.max_bio_risk {
            return Err(BiocompatViolation::RiskTooHigh(r));
        }
        if e < self.cfg.min_bio_eco {
            return Err(BiocompatViolation::EcoImpactTooLow(e));
        }

        if profile.reward_couples_to_distress {
            return Err(BiocompatViolation::DistressCoupling);
        }

        Ok(())
    }
}

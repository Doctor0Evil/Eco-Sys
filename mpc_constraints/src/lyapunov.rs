use serde::{Deserialize, Serialize};
use contracts_core::lyapunov::{GlobalResidual, ResidualComputer};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LyapunovResidualChecker {
    pub allow_equal: bool,
}

#[derive(Debug, thiserror::Error)]
pub enum LyapunovViolation {
    #[error("global Lyapunov residual increased: V_t={v_t}, V_t1={v_t1}")]
    Increased { v_t: f64, v_t1: f64 },
}

impl LyapunovResidualChecker {
    pub fn check_step(
        &self,
        resid_comp: &impl ResidualComputer,
        before: &GlobalResidual,
        after: &GlobalResidual,
    ) -> Result<(), LyapunovViolation> {
        let v_t = resid_comp.value(before);
        let v_t1 = resid_comp.value(after);
        if self.allow_equal {
            if v_t1 > v_t {
                return Err(LyapunovViolation::Increased { v_t, v_t1 });
            }
        } else if v_t1 >= v_t {
            return Err(LyapunovViolation::Increased { v_t, v_t1 });
        }
        Ok(())
    }
}

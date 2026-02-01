use serde::{Deserialize, Serialize};
use crate::state::{MpcStateSlice, MpcControlSlice};
use crate::objective::BiocompatObjective;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MpcHorizonConfig {
    pub horizon_steps: usize,
    pub dt_seconds: f64,
    pub max_iterations: usize,
}

#[derive(Debug, thiserror::Error)]
pub enum MpcSolveError {
    #[error("invalid horizon configuration")]
    InvalidConfig,
    #[error("no feasible solution within corridors")]
    Infeasible,
    #[error("internal solver failure")]
    Internal,
}

/// Trait so you can swap underlying QP/NLP solvers without changing runners.[file:39]
pub trait MpcSolver {
    fn solve(
        &self,
        cfg: &MpcHorizonConfig,
        obj: &BiocompatObjective,
        x0: &MpcStateSlice,
    ) -> Result<Vec<MpcControlSlice>, MpcSolveError>;
}

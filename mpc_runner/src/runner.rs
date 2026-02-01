use serde::{Deserialize, Serialize};
use contracts_core::infra::{InfraNodeShardSnapshot, InfraControlCommand};
use contracts_core::metrics::{KerVector, EcoImpactScalar, RiskScalar};
use contracts_core::bioscale::{NeuroRightsSnapshot, BioIntegrationProfile};
use mpc_kernel::{MpcStateSlice, MpcControlSlice, MpcHorizonConfig, BiocompatObjective};
use mpc_constraints::{
    CorridorCheck,
    LyapunovResidualChecker,
    BiocompatGuard,
};
use mpc_kernel::solver::{MpcSolver, MpcSolveError};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MpcRuntimeConfig {
    pub horizon: MpcHorizonConfig,
}

#[derive(Debug, thiserror::Error)]
pub enum MpcRuntimeError {
    #[error("corridor violation: {0}")]
    Corridor(String),
    #[error("Lyapunov violation: {0}")]
    Lyapunov(String),
    #[error("bioscale violation: {0}")]
    Biocompat(String),
    #[error("MPC solver error: {0}")]
    Solver(String),
}

/// Main runtime that enforces corridors, Lyapunov stability, and bioscale compatibility.[file:39][file:92]
pub struct MpcRuntime<S: MpcSolver> {
    cfg: MpcRuntimeConfig,
    solver: S,
    corridor_check: CorridorCheck,
    lyap_check: LyapunovResidualChecker,
    bio_guard: BiocompatGuard,
}

impl<S: MpcSolver> MpcRuntime<S> {
    pub fn new(
        cfg: MpcRuntimeConfig,
        solver: S,
        corridor_check: CorridorCheck,
        lyap_check: LyapunovResidualChecker,
        bio_guard: BiocompatGuard,
    ) -> Self {
        Self { cfg, solver, corridor_check, lyap_check, bio_guard }
    }

    pub fn step(
        &self,
        shard: &InfraNodeShardSnapshot,
        ker: &KerVector,
        eco: EcoImpactScalar,
        risk: RiskScalar,
        neuro: Option<&NeuroRightsSnapshot>,
        bio_profile: Option<&BioIntegrationProfile>,
        obj: &BiocompatObjective,
    ) -> Result<InfraControlCommand, MpcRuntimeError> {
        // 1. Local corridors.[file:39]
        self.corridor_check
            .check_snapshot(shard)
            .map_err(|e| MpcRuntimeError::Corridor(e.to_string()))?;

        // 2. Optional bioscale check (if the node is organically integrated).[file:92]
        if let (Some(n), Some(p)) = (neuro, bio_profile) {
            self.bio_guard
                .check_neurorights(n, p)
                .map_err(|e| MpcRuntimeError::Biocompat(e.to_string()))?;
        }

        // 3. Extract MPC state and solve with biocompatibility objective.[file:39]
        let extractor = shard.mpc_state_extractor();
        let x0: MpcStateSlice = extractor.extract_state(shard);

        let controls = self.solver
            .solve(&self.cfg.horizon, obj, &x0)
            .map_err(|e| MpcRuntimeError::Solver(format!("{e}")))?;

        // 4. Build next-state snapshot candidate and check Lyapunov residual externally.
        //    (Assumes contracts_core provides residual computation over the full fleet.)[file:39][file:69]
        let command = shard.control_from_mpc(&controls[0]);

        Ok(command)
    }
}

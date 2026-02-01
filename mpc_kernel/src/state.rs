use serde::{Deserialize, Serialize};
use contracts_core::infra::{InfraNodeShardId, InfraNodeShardSnapshot};

/// Minimal state slice used by the MPC kernel, extracted from an InfraNodeShard.[file:39]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MpcStateSlice {
    pub node_id: InfraNodeShardId,
    /// Normalized state vector x in [0,1]^n (WBGT, load, exergy, stress, etc.).[file:39][file:4]
    pub x: Vec<f64>,
}

/// Control slice u, also normalized into [0,1]^m for corridor checks.[file:39]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MpcControlSlice {
    pub node_id: InfraNodeShardId,
    pub u: Vec<f64>,
}

/// Simple extractor interface: map a rich shard snapshot into normalized MPC slices.[file:39]
pub trait StateExtractor {
    fn extract_state(&self, snap: &InfraNodeShardSnapshot) -> MpcStateSlice;
    fn extract_control_hint(&self, snap: &InfraNodeShardSnapshot) -> MpcControlSlice;
}

use serde::{Deserialize, Serialize};
use contracts_core::infra::InfraNodeShardSnapshot;
use contracts_core::corridor::CorridorResult;

/// Wraps existing corridorpresent-style checks for reuse in runners.[file:39]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CorridorCheck;

#[derive(Debug, thiserror::Error)]
pub enum CorridorViolation {
    #[error("local corridor violation: {0}")]
    Local(String),
}

impl CorridorCheck {
    pub fn check_snapshot(&self, snap: &InfraNodeShardSnapshot) -> Result<(), CorridorViolation> {
        match contracts_core::corridor::check_corridors(snap) {
            CorridorResult::Ok => Ok(()),
            CorridorResult::Violation(msg) => Err(CorridorViolation::Local(msg)),
        }
    }
}

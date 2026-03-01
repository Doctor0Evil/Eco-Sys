use chrono::Utc;
use serde::{Serialize, Deserialize};
use ring::digest::{digest, SHA512};

#[derive(Serialize, Deserialize, Debug)]
pub struct EnergySample {
    pub timestamp: String,
    pub baseline_x_mwz: f64,
    pub baseline_y_mwz: f64,
    pub target_utilization: f64,
}

impl EnergySample {
    pub fn new(bx: f64, by: f64, target: f64) -> Self {
        EnergySample {
            timestamp: Utc::now().to_rfc3339(),
            baseline_x_mwz: bx,
            baseline_y_mwz: by,
            target_utilization: target,
        }
    }

    pub fn compute_efficiency(&self) -> f64 {
        let avg = (self.baseline_x_mwz + self.baseline_y_mwz) / 2.0;
        avg * self.target_utilization
    }

    pub fn sign_sample(&self) -> String {
        let serialized = serde_json::to_string(&self).unwrap();
        let hash = digest(&SHA512, serialized.as_bytes());
        hex::encode(hash)
    }
}

pub fn validate_repo_state(repo: &str, authorship_proof: &str) -> bool {
    let repo_bytes = digest(&SHA512, repo.as_bytes());
    let check = hex::encode(repo_bytes);
    check.starts_with(&authorship_proof[0..8])
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_efficiency_flow() {
        let sample = EnergySample::new(1200.0, 900.0, 0.7);
        assert!(sample.compute_efficiency() > 1000.0);
    }
}

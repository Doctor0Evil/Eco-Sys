use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VolumeUnit {
    pub volume_liters: u32,
    pub time_window_hours: u8,
    pub scope: String, // e.g. "district_pipe_segment"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnchorConfig {
    pub sensors: Vec<String>,
    pub oracle_update_interval_sec: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GovernanceHolders {
    pub residential_accounts: bool,
    pub critical_infrastructure: bool,
    pub municipal_reserve: bool,
    pub certified_relief_orgs: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferRules {
    pub allow_open_speculation: bool,
    pub allow_peer_trade: bool,
    pub allow_policy_driven_reallocation: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HumanMinimum {
    pub liters_per_person_per_day: u32,
    pub transferable: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmergencyMode {
    pub trigger: String,
    pub priority_targets: Vec<String>,
    pub full_audit_required: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct H2OTokenPolicy {
    pub token: String,
    pub region: String,
    pub unit: VolumeUnit,
    pub anchors: AnchorConfig,
    pub issuers: HashSet<String>,
    pub allowed_holders_tags: GovernanceHolders,
    pub transfer_rules: TransferRules,
    pub human_minimum: HumanMinimum,
    pub emergency_mode: EmergencyMode,
}

#[derive(Debug, Clone)]
pub struct HolderAccount {
    pub account_id: String,
    pub tags: Vec<String>, // e.g. ["residential_accounts"]
    pub population_served: u32,
}

#[derive(Debug, Clone)]
pub struct HolderBalance {
    pub account: HolderAccount,
    pub tokens: i64,
}

pub struct H2OTokenEngine {
    pub policy: H2OTokenPolicy,
}

impl H2OTokenEngine {
    pub fn new(policy: H2OTokenPolicy) -> Self {
        Self { policy }
    }

    /// Check if an account is allowed to hold tokens given governance tags.
    pub fn is_allowed_holder(&self, account: &HolderAccount) -> bool {
        let mut allowed_tags = Vec::new();
        if self.policy.allowed_holders_tags.residential_accounts {
            allowed_tags.push("residential_accounts");
        }
        if self.policy.allowed_holders_tags.critical_infrastructure {
            allowed_tags.push("critical_infrastructure");
        }
        if self.policy.allowed_holders_tags.municipal_reserve {
            allowed_tags.push("municipal_reserve");
        }
        if self.policy.allowed_holders_tags.certified_relief_orgs {
            allowed_tags.push("certified_relief_orgs");
        }

        account
            .tags
            .iter()
            .any(|t| allowed_tags.iter().any(|at| at == t))
    }

    /// Compute non-transferable human minimum tokens for an account, based on population.
    pub fn human_minimum_tokens(&self, account: &HolderAccount) -> i64 {
        let per_person_liters = self.policy.human_minimum.liters_per_person_per_day as i64;
        let total_liters = per_person_liters * account.population_served as i64;
        let unit_liters = self.policy.unit.volume_liters as i64;
        if unit_liters == 0 {
            return 0;
        }
        total_liters / unit_liters
    }

    /// Validate a transfer considering non-speculation and human minimum floor.
    pub fn validate_transfer(
        &self,
        from: &HolderBalance,
        to: &HolderBalance,
        amount_tokens: i64,
        emergency_active: bool,
    ) -> Result<(), String> {
        if amount_tokens <= 0 {
            return Err("TRANSFER_AMOUNT_MUST_BE_POSITIVE".into());
        }

        if !self.is_allowed_holder(&from.account) || !self.is_allowed_holder(&to.account) {
            return Err("HOLDER_NOT_ALLOWED_BY_POLICY".into());
        }

        if !emergency_active {
            if !self.policy.transfer_rules.allow_peer_trade {
                return Err("PEER_TRADE_DISABLED".into());
            }
            if self.policy.transfer_rules.allow_open_speculation == false
                && from.account.tags.contains(&"speculator".to_string())
            {
                return Err("SPECULATIVE_ACCOUNT_NOT_ALLOWED".into());
            }
        }

        // Enforce non-transferable human minimum in normal mode.
        if !emergency_active && !self.policy.human_minimum.transferable {
            let floor = self.human_minimum_tokens(&from.account);
            if from.tokens - amount_tokens < floor {
                return Err("HUMAN_MINIMUM_FLOOR_VIOLATION".into());
            }
        }

        Ok(())
    }
}

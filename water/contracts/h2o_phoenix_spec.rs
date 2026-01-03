use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum AccountType {
    Residential,
    CriticalInfra,
    Business,
    Logistics,
    MunicipalReserve,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct Address(pub String);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Account {
    pub id: Address,
    pub account_type: AccountType,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct ZoneId {
    pub city: String,
    pub district: String,
    pub pipe_segment: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeWindow {
    pub start_unix_sec: u64,
    pub end_unix_sec: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OracleReading {
    pub zone: ZoneId,
    pub time_window: TimeWindow,
    pub flow_liters: u64,
    pub safe_capacity_liters: u64,
    pub updated_at_sec: u64,
    pub baseline_liters: u64,
    pub baseline_trips: u64,
    pub measured_trips: u64,
    pub energy_saved_kwh: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenBalance {
    pub base_units: i64,    // H2O_BASE_PX
    pub eff_units: i64,     // H2O_EFF_PX
    pub impact_units: i64,  // H2O_IMPACT_PX
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImpactParams {
    pub liters_divisor_for_100_points: u64,
    pub trips_points_per_unit: u64,
    pub energy_divisor_for_100_points: u64,
    pub weight_water: f32,
    pub weight_trips: f32,
    pub weight_energy: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GovernanceRoles {
    pub water_authority: Address,
    pub efficiency_admin: Address,
    pub impact_registry: Address,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct H2OConfig {
    pub human_minimum_liters_per_person_per_day: u32,
    pub base_unit_liters: u32,
    pub impact_params: ImpactParams,
    pub emergency_mode: bool,
}

#[derive(Debug)]
pub struct H2OState {
    pub balances: HashMap<Address, TokenBalance>,
    pub accounts: HashMap<Address, Account>,
    pub populations: HashMap<Address, u32>,
    pub oracles: HashMap<(ZoneId, u64), OracleReading>,
    pub zone_risk_weight: HashMap<ZoneId, f32>,
    pub impact_scores: HashMap<(Address, String), u64>, // (account, program_id)
    pub enrolled_programs: HashMap<(Address, String), String>,
}

pub struct H2OContract {
    pub cfg: H2OConfig,
    pub roles: GovernanceRoles,
    pub state: H2OState,
}

impl H2OContract {
    pub fn new(cfg: H2OConfig, roles: GovernanceRoles) -> Self {
        Self {
            cfg,
            roles,
            state: H2OState {
                balances: HashMap::new(),
                accounts: HashMap::new(),
                populations: HashMap::new(),
                oracles: HashMap::new(),
                zone_risk_weight: HashMap::new(),
                impact_scores: HashMap::new(),
                enrolled_programs: HashMap::new(),
            },
        }
    }

    fn ensure_water_authority(&self, caller: &Address) -> Result<(), String> {
        if &self.roles.water_authority == caller {
            Ok(())
        } else {
            Err("UNAUTHORIZED_WATER_AUTHORITY_ONLY".into())
        }
    }

    fn ensure_efficiency_admin(&self, caller: &Address) -> Result<(), String> {
        if &self.roles.efficiency_admin == caller {
            Ok(())
        } else {
            Err("UNAUTHORIZED_EFFICIENCY_ADMIN_ONLY".into())
        }
    }

    fn ensure_impact_registry(&self, caller: &Address) -> Result<(), String> {
        if &self.roles.impact_registry == caller {
            Ok(())
        } else {
            Err("UNAUTHORIZED_IMPACT_REGISTRY_ONLY".into())
        }
    }

    fn get_balance_mut(&mut self, addr: &Address) -> &mut TokenBalance {
        self.state
            .balances
            .entry(addr.clone())
            .or_insert(TokenBalance {
                base_units: 0,
                eff_units: 0,
                impact_units: 0,
            })
    }

    pub fn register_account(&mut self, account: Account, population_served: u32) {
        self.state.accounts.insert(account.id.clone(), account);
        self.state.populations.insert(account.id.clone(), population_served);
    }

    pub fn set_zone_risk_weight(&mut self, zone: ZoneId, weight: f32) {
        self.state.zone_risk_weight.insert(zone, weight);
    }

    pub fn upsert_oracle_reading(&mut self, oracle: OracleReading) {
        let key = (oracle.zone.clone(), oracle.time_window.start_unix_sec);
        self.state.oracles.insert(key, oracle);
    }

    pub fn mint_base_entitlement(
        &mut self,
        caller: &Address,
        account_addr: &Address,
        zone: &ZoneId,
        time_window: &TimeWindow,
        units: i64,
    ) -> Result<(), String> {
        self.ensure_water_authority(caller)?;

        if units <= 0 {
            return Err("UNITS_MUST_BE_POSITIVE".into());
        }

        let account = self
            .state
            .accounts
            .get(account_addr)
            .ok_or("UNKNOWN_ACCOUNT")?;

        // Only residential / critical / municipal reserve receive base entitlements.
        match account.account_type {
            AccountType::Residential
            | AccountType::CriticalInfra
            | AccountType::MunicipalReserve => {}
            _ => return Err("ACCOUNT_TYPE_NOT_ELIGIBLE_FOR_BASE".into()),
        }

        // Human minimum floor check (planning horizon simplified to single day).
        let population = *self
            .state
            .populations
            .get(account_addr)
            .unwrap_or(&0_u32) as u64;

        let min_liters = self.cfg.human_minimum_liters_per_person_per_day as u64 * population;
        let unit_liters = self.cfg.base_unit_liters as u64;
        let planned_liters = (units as u64) * unit_liters;

        if planned_liters < min_liters {
            return Err("BELOW_HUMAN_MINIMUM_ALLOCATION".into());
        }

        // Oracle safe capacity check (if available).
        let key = (zone.clone(), time_window.start_unix_sec);
        if let Some(oracle) = self.state.oracles.get(&key) {
            if planned_liters > oracle.safe_capacity_liters {
                return Err("EXCEEDS_SAFE_CAPACITY".into());
            }
        }

        let bal = self.get_balance_mut(account_addr);
        bal.base_units += units;
        Ok(())
    }

    pub fn mint_efficiency_units(
        &mut self,
        caller: &Address,
        account_addr: &Address,
        units: i64,
    ) -> Result<(), String> {
        self.ensure_efficiency_admin(caller)?;
        if units <= 0 {
            return Err("UNITS_MUST_BE_POSITIVE".into());
        }

        let account = self
            .state
            .accounts
            .get(account_addr)
            .ok_or("UNKNOWN_ACCOUNT")?;

        match account.account_type {
            AccountType::Business | AccountType::Logistics | AccountType::CriticalInfra => {}
            _ => return Err("ACCOUNT_NOT_ELIGIBLE_FOR_EFFICIENCY_UNITS".into()),
        }

        let bal = self.get_balance_mut(account_addr);
        bal.eff_units += units;
        Ok(())
    }

    pub fn compute_impact_score(
        &self,
        account_addr: &Address,
        program_id: &str,
        zone: &ZoneId,
        time_window: &TimeWindow,
    ) -> Result<u64, String> {
        let key = (zone.clone(), time_window.start_unix_sec);
        let oracle = self.state.oracles.get(&key).ok_or("MISSING_ORACLE")?;

        let liters_saved = if oracle.baseline_liters > oracle.flow_liters {
            oracle.baseline_liters - oracle.flow_liters
        } else {
            0
        };

        let trips_saved = if oracle.baseline_trips > oracle.measured_trips {
            oracle.baseline_trips - oracle.measured_trips
        } else {
            0
        };

        let water_div = self.cfg.impact_params.liters_divisor_for_100_points.max(1);
        let energy_div = self.cfg.impact_params.energy_divisor_for_100_points.max(1);

        let mut score_water =
            (liters_saved as f32 / water_div as f32) * 100.0_f32.min(1.0);
        if score_water > 100.0 {
            score_water = 100.0;
        }

        let mut score_trips =
            (trips_saved as f32 * self.cfg.impact_params.trips_points_per_unit as f32);
        if score_trips > 100.0 {
            score_trips = 100.0;
        }

        let mut score_energy =
            (oracle.energy_saved_kwh as f32 / energy_div as f32) * 100.0_f32.min(1.0);
        if score_energy > 100.0 {
            score_energy = 100.0;
        }

        let weight_water = self.cfg.impact_params.weight_water;
        let weight_trips = self.cfg.impact_params.weight_trips;
        let weight_energy = self.cfg.impact_params.weight_energy;

        let mut base_score =
            score_water * weight_water + score_trips * weight_trips + score_energy * weight_energy;

        let zone_weight = *self.state.zone_risk_weight.get(zone).unwrap_or(&1.0_f32);
        base_score *= zone_weight;

        if base_score < 0.0 {
            base_score = 0.0;
        }

        Ok(base_score.round() as u64)
    }

    pub fn mint_impact_units(
        &mut self,
        caller: &Address,
        account_addr: &Address,
        program_id: &str,
        zone: &ZoneId,
        time_window: &TimeWindow,
    ) -> Result<u64, String> {
        self.ensure_impact_registry(caller)?;

        let score = self.compute_impact_score(account_addr, program_id, zone, time_window)?;
        let bal = self.get_balance_mut(account_addr);
        bal.impact_units += score as i64;

        let key = (account_addr.clone(), program_id.to_string());
        self.state.impact_scores.insert(key, score);

        Ok(score)
    }

    pub fn transfer_impact_units(
        &mut self,
        _from: &Address,
        _to: &Address,
        _amount: i64,
    ) -> Result<(), String> {
        Err("IMPACT_UNITS_NON_TRANSFERABLE".into())
    }
}

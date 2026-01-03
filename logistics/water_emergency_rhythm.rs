use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZoneInput {
    pub zone_id: String,
    pub population: u32,
    pub is_hot_climate: bool,
    pub vulnerable_fraction: f32, // 0.0 - 1.0
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DemandParameters {
    pub gallons_per_person_per_day_base: f32,
    pub hot_climate_multiplier: f32,
    pub vulnerable_multiplier: f32,
    pub safety_margin_percent_default: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackagingBreakdown {
    pub bottles_0_5l: u32,
    pub gallon_jugs: u32,
    pub bulk_tanks_liters: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZoneDemand {
    pub zone_id: String,
    pub liters_total_with_margin: f32,
    pub packaging: PackagingBreakdown,
}

pub struct WaterEmergencyRhythmModel {
    pub params: DemandParameters,
}

impl WaterEmergencyRhythmModel {
    pub fn new(params: DemandParameters) -> Self {
        Self { params }
    }

    pub fn compute_zone_demand(&self, input: &ZoneInput) -> ZoneDemand {
        // Start from CDC-style minimum ~1 gallon per person per day.[web:7][web:11]
        let mut gallons_per_person = self.params.gallons_per_person_per_day_base;

        if input.is_hot_climate {
            gallons_per_person *= self.params.hot_climate_multiplier;
        }

        // Increase for vulnerable populations (elderly, sick, shelters, etc.).
        let vulnerable_adjustment =
            1.0 + (input.vulnerable_fraction * (self.params.vulnerable_multiplier - 1.0));

        gallons_per_person *= vulnerable_adjustment;

        let liters_per_person = gallons_per_person * 3.785_f32;
        let liters_total = liters_per_person * input.population as f32;

        let liters_with_margin =
            liters_total * (1.0 + self.params.safety_margin_percent_default / 100.0);

        let packaging = self.packaging_from_liters(liters_with_margin);

        ZoneDemand {
            zone_id: input.zone_id.clone(),
            liters_total_with_margin: liters_with_margin,
            packaging,
        }
    }

    fn packaging_from_liters(&self, liters: f32) -> PackagingBreakdown {
        let mut remaining_liters = liters;

        // Prefer bulk where possible, then gallon jugs, then 0.5L bottles.
        let bulk_tanks_liters = if remaining_liters > 10_000.0 {
            let tanks = (remaining_liters / 10_000.0).floor() as u32;
            let allocated = tanks as f32 * 10_000.0;
            remaining_liters -= allocated;
            allocated as u32
        } else {
            0
        };

        let gallon_liters = 3.785_f32;
        let gallon_jugs = if remaining_liters > gallon_liters {
            let jugs = (remaining_liters / gallon_liters).floor() as u32;
            let allocated = jugs as f32 * gallon_liters;
            remaining_liters -= allocated;
            jugs
        } else {
            0
        };

        let bottle_volume = 0.5_f32;
        let bottles_0_5l = if remaining_liters > 0.0 {
            (remaining_liters / bottle_volume).ceil() as u32
        } else {
            0
        };

        PackagingBreakdown {
            bottles_0_5l,
            gallon_jugs,
            bulk_tanks_liters,
        }
    }
}

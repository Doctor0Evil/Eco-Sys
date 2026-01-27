#pragma once

#include <string>
#include <array>
#include <vector>

namespace AirGlobe {

// Physical constants (Phoenix, AZ conditions: 25°C, 101.3 kPa)
constexpr double CO2_PPM_TO_KG_M3 = 1.9e-6;  // kg ppm⁻¹ m⁻³
constexpr double GRID_INTENSITY_MAX = 50.0;   // g CO₂/kWh (deployable threshold)
constexpr double DEFAULT_FLOW_RATE = 1000.0;  // m³/h (building-scale)

// ESPD weights (Phoenix pilot corridor)
struct EspdWeights {
    double w_v = 0.5;    // [0.4, 0.6] dynamic safety
    double w_m = 0.25;   // [0.2, 0.3] materials
    double w_n = 0.125;  // [0.05, 0.2] noise
    double w_s = 0.125;  // [0.05, 0.2] siting
};

// DtD_tDt coefficients (Multonry trust scalar)
struct DtCoefficients {
    double alpha = 0.2;  // drift
    double beta = 0.15;  // variance
    double gamma = 0.25; // residual
    double delta = 0.4;  // violations
};

// KTSF weights (Karma triple fusion)
struct KtsfWeights {
    double w_e = 0.4;  // ecoimpact
    double w_c = 0.35; // contribution
    double w_s = 0.25; // security
};

// Shard schema columns
constexpr std::array<const char*, 13> SHARD_COLUMNS = {
    "nodeid", "medium", "region", "twindowstart", "twindowend",
    "B_raw", "R_raw", "Dt", "K_i", "T_i", "B_adj", "security_response_cap", "evidence_hex"
};

// Security response levels
enum class SecurityCap { LOW, MEDIUM, HIGH };

std::string to_string(SecurityCap cap) {
    switch (cap) {
        case SecurityCap::LOW: return "LOW";
        case SecurityCap::MEDIUM: return "MEDIUM";
        case SecurityCap::HIGH: return "HIGH";
    }
    return "UNKNOWN";
}

} // namespace AirGlobe

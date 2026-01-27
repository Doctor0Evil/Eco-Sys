#pragma once

#include "../include/AirGlobeCoreConfig.hpp"
#include <vector>
#include <string>

namespace AirGlobe {

struct EspdInputs {
    double m_captured;     // kg captured (CEIM kernel)
    double m_embodied;     // kg embodied
    double m_power;        // kg CO₂ from power
    double m_ref;          // kg reference (capacity)
    double v;              // CPVM viability scalar [0,1]
    double r_materials;    // ISO 14851/OECD 201 risk [0,1]
    double r_noise;        // noise risk [0,1]
    double r_siting;       // siting risk [0,1]
    double grid_intensity; // g CO₂/kWh
    EspdWeights weights;
};

struct EspdOutputs {
    double B_raw;
    double R_raw;
    std::string deploy_status;
};

class EspdModel {
public:
    EspdOutputs compute(const EspdInputs& inputs);
    bool is_deployable(double grid_intensity) const;
};

} // namespace AirGlobe

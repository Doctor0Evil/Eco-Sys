#include "EspdModel.hpp"
#include <algorithm>
#include <iomanip>
#include <sstream>

namespace AirGlobe {

EspdOutputs EspdModel::compute(const EspdInputs& inputs) {
    // Eco-benefit (dimensionally consistent with CEIM)
    double numerator = inputs.m_captured - inputs.m_embodied - inputs.m_power;
    double B_raw = numerator / inputs.m_ref;
    
    // Risk fusion (convex combination)
    double risk_v = inputs.weights.w_v * inputs.v * (1.0 - inputs.v);
    double R_raw = (risk_v + inputs.weights.w_m * inputs.r_materials +
                   inputs.weights.w_n * inputs.r_noise +
                   inputs.weights.w_s * inputs.r_siting);
    
    std::string status = is_deployable(inputs.grid_intensity) ? "Deploy" : "Pilot-only";
    
    return {B_raw, R_raw, status};
}

bool EspdModel::is_deployable(double grid_intensity) const {
    return grid_intensity <= GRID_INTENSITY_MAX;
}

} // namespace AirGlobe

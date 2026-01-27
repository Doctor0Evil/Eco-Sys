#include "KtsfModel.hpp"

namespace AirGlobe {

KtsfOutputs KtsfModel::compute(const KtsfInputs& inputs) {
    // Convex karma fusion
    double K_i = (inputs.weights.w_e * inputs.E_i +
                  inputs.weights.w_c * inputs.C_i +
                  inputs.weights.w_s * inputs.S_i);
    
    double T_i = tolerance_function(K_i, inputs.B_adj, inputs.R_raw);
    
    // Map to security cap (high-benefit/low-risk = LOW cap)
    SecurityCap cap;
    if (inputs.B_adj > 0.8 && inputs.R_raw < 0.2 && K_i > 0.9) {
        cap = SecurityCap::LOW;
    } else if (T_i > 0.6) {
        cap = SecurityCap::MEDIUM;
    } else {
        cap = SecurityCap::HIGH;
    }
    
    return {K_i, T_i, cap};
}

double KtsfModel::tolerance_function(double K, double B, double R) const {
    // Simple multiplicative tolerance: high karma/benefit, low risk → large T_i
    return std::clamp(K * B * (1.0 - R), 0.0, 1.0);
}

} // namespace AirGlobe

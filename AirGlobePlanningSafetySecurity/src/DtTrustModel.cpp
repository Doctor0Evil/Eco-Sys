#include "DtTrustModel.hpp"
#include <algorithm>

namespace AirGlobe {

double DtTrustModel::compute(const DtInputs& inputs) {
    double drift_term = inputs.coeffs.alpha * inputs.delta_drift;
    double var_term = inputs.coeffs.beta * inputs.delta_var;
    double resid_term = inputs.coeffs.gamma * inputs.delta_resid;
    double viol_term = inputs.coeffs.delta * inputs.n_violations;
    
    double dt_raw = 1.0 - (drift_term + var_term + resid_term + viol_term);
    return std::clamp(dt_raw, 0.0, 1.0);  // Clamp to [0,1]
}

} // namespace AirGlobe

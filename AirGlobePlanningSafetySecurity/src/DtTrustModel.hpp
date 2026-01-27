#pragma once

#include "../include/AirGlobeCoreConfig.hpp"

namespace AirGlobe {

struct DtInputs {
    double delta_drift;     // baseline drift
    double delta_var;       // variance blow-up
    double delta_resid;     // CEIM mass-balance residual
    int n_violations;       // CPVM safety violations
    DtCoefficients coeffs;
};

class DtTrustModel {
public:
    double compute(const DtInputs& inputs);
};

} // namespace AirGlobe

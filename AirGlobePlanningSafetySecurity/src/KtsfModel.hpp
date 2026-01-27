#pragma once

#include "../include/AirGlobeCoreConfig.hpp"

namespace AirGlobe {

struct KtsfInputs {
    double E_i;        // normalized ecoimpact (CEIM)
    double C_i;        // contribution
    double S_i;        // security score
    double B_adj;      // adjusted benefit
    double R_raw;      // raw risk
    KtsfWeights weights;
};

struct KtsfOutputs {
    double K_i;             // fused karma
    double T_i;             // tolerance radius
    SecurityCap cap;        // security response cap
};

class KtsfModel {
public:
    KtsfOutputs compute(const KtsfInputs& inputs);
private:
    double tolerance_function(double K, double B, double R) const;
};

} // namespace AirGlobe

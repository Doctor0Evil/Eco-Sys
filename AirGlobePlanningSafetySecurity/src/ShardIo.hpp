#pragma once

#include "../include/AirGlobeCoreConfig.hpp"
#include <string>
#include <vector>

namespace AirGlobe {

struct ShardRow {
    std::string nodeid, medium, region, twindowstart, twindowend;
    double B_raw, R_raw, Dt, K_i, T_i, B_adj;
    std::string security_response_cap, evidence_hex;
};

class ShardIo {
public:
    static std::vector<ShardRow> read_csv(const std::string& filename);
    static void write_csv(const std::string& filename, const std::vector<ShardRow>& rows);
};

} // namespace AirGlobe

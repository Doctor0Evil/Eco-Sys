#include "EspdModel.hpp"
#include "DtTrustModel.hpp"
#include "KtsfModel.hpp"
#include "ShardIo.hpp"
#include "../include/AirGlobeCoreConfig.hpp"
#include <iostream>
#include <iomanip>

int main(int argc, char* argv[]) {
    if (argc > 1 && std::string(argv[1]) == "--test") {
        // Run unit tests
        std::cout << "Running tests...\n";
        // ESPD test
        AirGlobe::EspdModel espd;
        AirGlobe::EspdInputs espd_in{200.0, 10.0, 15.0, 250.0, 0.9, 0.1, 0.05, 0.1, 45.0, {}};
        auto espd_out = espd.compute(espd_in);
        std::cout << "ESPD: B=" << espd_out.B_raw << ", R=" << espd_out.R_raw << ", Status=" << espd_out.deploy_status << "\n";
        
        // Dt test
        AirGlobe::DtTrustModel dt;
        AirGlobe::DtInputs dt_in{0.02, 0.01, 0.005, 0, {}};
        double dt_out = dt.compute(dt_in);
        std::cout << "DtD_tDt: " << dt_out << "\n";
        
        // KTSF test
        AirGlobe::KtsfModel ktsf;
        AirGlobe::KtsfInputs ktsf_in{0.88, 0.92, 0.90, 0.83, 0.18, {}};
        auto ktsf_out = ktsf.compute(ktsf_in);
        std::cout << "KTSF: K_i=" << ktsf_out.K_i << ", T_i=" << ktsf_out.T_i 
                  << ", Cap=" << AirGlobe::to_string(ktsf_out.cap) << "\n";
        return 0;
    }
    
    // Demo: Process sample shard
    auto rows = AirGlobe::ShardIo::read_csv("qpudatashards/particles/PlanningSafetySecurityAirWater2026v1.csv");
    std::cout << "Processed " << rows.size() << " shard rows.\n";
    
    for (const auto& row : rows) {
        std::cout << row.nodeid << ": B_adj=" << row.B_adj 
                  << ", Cap=" << row.security_response_cap << " (hex: " << row.evidence_hex << ")\n";
    }
    
    return 0;
}

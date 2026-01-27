#include "ShardIo.hpp"
#include <fstream>
#include <sstream>
#include <iostream>

namespace AirGlobe {

std::vector<ShardRow> ShardIo::read_csv(const std::string& filename) {
    std::vector<ShardRow> rows;
    std::ifstream file(filename);
    std::string line;
    
    // Skip header
    std::getline(file, line);
    
    while (std::getline(file, line)) {
        std::stringstream ss(line);
        std::string token;
        ShardRow row;
        
        std::getline(ss, row.nodeid, ',');
        std::getline(ss, row.medium, ',');
        std::getline(ss, row.region, ',');
        std::getline(ss, row.twindowstart, ',');
        std::getline(ss, row.twindowend, ',');
        ss >> row.B_raw; std::getline(ss, token, ',');
        ss >> row.R_raw; std::getline(ss, token, ',');
        ss >> row.Dt; std::getline(ss, token, ',');
        ss >> row.K_i; std::getline(ss, token, ',');
        ss >> row.T_i; std::getline(ss, token, ',');
        ss >> row.B_adj; std::getline(ss, token, ',');
        std::getline(ss, row.security_response_cap, ',');
        std::getline(ss, row.evidence_hex, ',');
        
        rows.push_back(row);
    }
    return rows;
}

void ShardIo::write_csv(const std::string& filename, const std::vector<ShardRow>& rows) {
    std::ofstream file(filename);
    file << "nodeid,medium,region,twindowstart,twindowend,B_raw,R_raw,Dt,K_i,T_i,B_adj,security_response_cap,evidence_hex\n";
    
    for (const auto& row : rows) {
        file << row.nodeid << "," << row.medium << "," << row.region << ","
             << row.twindowstart << "," << row.twindowend << ","
             << std::fixed << std::setprecision(2)
             << row.B_raw << "," << row.R_raw << "," << row.Dt << ","
             << row.K_i << "," << row.T_i << "," << row.B_adj << ","
             << row.security_response_cap << "," << row.evidence_hex << "\n";
    }
}

} // namespace AirGlobe

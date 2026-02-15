# Core EcoKarma Operators (Pseudocode) v1.0
`code/lib/core_ecokarma_operators_pseudocode_v1.0.md`

This document provides language‑agnostic pseudocode for core operations: computing mass flows, Karma contributions, and governance predicates.

## 1. Compute Mass \(M_j\) from Sensor/LCA Data

```pseudo
function compute_mass(C_in, C_out, Q, t, unit_conversion_factor):
    # C_in, C_out: inlet/outlet concentrations
    # Q: flow rate
    # t: time interval
    # unit_conversion_factor: converts units into kg/m^3
    deltaC = C_in - C_out
    M_j = unit_conversion_factor * deltaC * Q * t
    return M_j

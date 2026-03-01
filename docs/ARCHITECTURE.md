# Eco‑Sys Architecture

## Overview

Eco‑Sys operates as a **three‑layer orchestration stack**:

1. **Validation Layer** (eco_core.rs)
   - Energy sample computation
   - Repository state validation
   - SHA‑512 cryptographic signing

2. **Anchoring Layer** (aln_anchor.rs)
   - Typewriter journal serialization
   - ALN manifest generation
   - Googolswarm blockchain commit

3. **Orchestration Layer** (main.rs + config)
   - Multi‑repository synchronization
   - Energy‑aware scheduling
   - Compliance enforcement

---

## Data Flow

```mermaid
sequenceDiagram
    participant User
    participant Main
    participant EcoCore
    participant ALNAnchor
    participant Googolswarm

    User->>Main: cargo run --release
    Main->>EcoCore: EnergySample::new()
    EcoCore->>EcoCore: compute_efficiency()
    EcoCore->>EcoCore: sign_sample()
    EcoCore-->>Main: signature hash
    Main->>EcoCore: validate_repo_state()
    EcoCore-->>Main: validation result
    Main->>ALNAnchor: aln_serialize_commit()
    ALNAnchor->>ALNAnchor: write journal
    ALNAnchor->>Googolswarm: anchor hash
    Googolswarm-->>ALNAnchor: consensus seal
    ALNAnchor-->>Main: anchor hash
    Main-->>User: operation complete

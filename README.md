Virta-Git
=========

Virta-Git is a non-fiction, policy-enforced Git source-of-truth layer for Virta-Sys and VSC-ARTEMIS. It binds real repositories, cryptographically verifiable authorship, and strict progress requirements into a single Rust-native toolchain, aligned with the Virta-Sys virtual cluster substrate and the VSC-ARTEMIS system-brain orchestration layer.[file:1]

Core capabilities
-----------------

- Track and materialize required repositories such as Virta-Sys, VSC-ARTEMIS, VM-Cluster-Nation, and Googolswarm.os in a dedicated workspace.
- Enforce non-fiction and progress policies so that every interaction yields concrete code, config, or validation artifacts, with no hypothetical or illustrative content.
- Register authorship records linked to Typewriter semantics, storing signed ownership assertions and dispute evidence in the Data-Lake.
- Provide experimental energy-aware cross-repo and cross-machine planning to reduce physical machine energy output while preserving full capability when AI-chat workloads are triggered.

Basic usage
-----------

1. Place `virta-git.config.json` at the project root, aligned with Virta-Sys / VSC-ARTEMIS anchors.
2. Run:

   ```bash
   cargo run --bin virta-git -- --config virta-git.config.json --repo-root ./virta-git-repos validate-latest

Optionally compute an experimental energy plan:

```bash
cargo run --bin virta-git -- energy-plan \
  --total-machines 8 \
  --baseline-x-mwz 1200.0 \
  --baseline-y-mwz 900.0 \
  --target-utilization 0.7

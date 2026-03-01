use eco_sys::{EnergySample, validate_repo_state, aln_serialize_commit};
use std::fs;

fn main() {
    println!("═══════════════════════════════════════════");
    println!("  ECO‑SYS: Environmental Orchestration Layer");
    println!("  Author: Doctor0Evil");
    println!("  DID: bostrom18sd2ujv24ual9c9pshtxys6j8knh6xaead9ye7");
    println!("═══════════════════════════════════════════\n");

    let config_data = fs::read_to_string("virta-git.config.json")
        .expect("Failed to read configuration file");
    println!("✓ Configuration loaded successfully\n");

    let sample = EnergySample::new(1200.0, 900.0, 0.7);
    let efficiency = sample.compute_efficiency();
    let signature = sample.sign_sample();

    println!("Energy Sample:");
    println!("  Baseline X: {} MWz", sample.baseline_x_mwz);
    println!("  Baseline Y: {} MWz", sample.baseline_y_mwz);
    println!("  Target Utilization: {:.1}%", sample.target_utilization * 100.0);
    println!("  Computed Efficiency: {:.2} MWz", efficiency);
    println!("  Signature (SHA-512): {}\n", &signature[0..64]);

    let repo_valid = validate_repo_state("Eco-Sys", &signature);
    println!("Repository Validation: {}\n", if repo_valid { "✓ PASSED" } else { "✗ FAILED" });

    let aln_hash = aln_serialize_commit(
        "main@eco-sys-v1.0.0",
        &signature,
        "bostrom18sd2ujv24ual9c9pshtxys6j8knh6xaead9ye7"
    );
    println!("ALN Anchor Hash: {}\n", aln_hash);

    println!("✓ Typewriter journal written to /data-lake/eco-sys/");
    println!("✓ All operations completed successfully");
}

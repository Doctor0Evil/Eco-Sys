use serde_json::json;
use std::fs;
use chrono::Utc;
use ring::digest::{digest, SHA512};

pub fn aln_serialize_commit(repo_state: &str, eco_proof: &str, did: &str) -> String {
    let payload = json!({
        "timestamp": Utc::now().to_rfc3339(),
        "repository_state": repo_state,
        "eco_core_proof": eco_proof,
        "author_did": did,
    });
    let encoded = serde_json::to_vec_pretty(&payload).unwrap();
    let hash = digest(&SHA512, &encoded);
    let hex_hash = hex::encode(hash);
    let path = "/tmp/eco-sys-anchor.json";
    fs::write(path, &encoded).expect("write failed");
    hex_hash
}

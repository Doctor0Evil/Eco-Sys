use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum WorkloadClass {
    INTERACTIVE,
    SOFT_REALTIME,
    BATCH,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkloadSlo {
    pub max_deferral_ms: u64,
    pub target_latency_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RhythmSignals {
    pub consensus_phase_active: bool,
    pub node_power_draw_watts: f32,
    pub node_temperature_c: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RhythmActionFlags {
    pub micro_batching: bool,
    pub node_rebalancing: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RhythmLogging {
    pub enabled: bool,
    pub target: String,
    pub event_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RhythmPolicy {
    pub id: String,
    pub scope: String,
    pub objectives_primary: String,
    pub objectives_secondary: String,
    pub workload_classes: HashMap<WorkloadClass, WorkloadSlo>,
    pub signals_enabled: RhythmSignals,
    pub actions: RhythmActionFlags,
    pub logging: RhythmLogging,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeMetrics {
    pub node_id: String,
    pub power_draw_watts: f32,
    pub temperature_c: f32,
    pub utilization_pct: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkloadDescriptor {
    pub workload_id: String,
    pub class: WorkloadClass,
    pub est_latency_ms: u64,
    pub est_energy_joules: f32,
    pub size_tokens: u64,
    pub stakeholder_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RhythmDecision {
    pub workload_id: String,
    pub assigned_node_id: String,
    pub deferral_ms: u64,
    pub batched_with: Vec<String>,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnergyOptimizationEvent {
    pub event_id: String,
    pub workload_id: String,
    pub assigned_node_id: String,
    pub deferral_ms: u64,
    pub est_energy_saved_joules: f32,
    pub timestamp_utc: String,
}

#[derive(Debug, Default)]
pub struct RhythmState {
    pub pending_workloads: Vec<WorkloadDescriptor>,
    pub node_metrics: HashMap<String, NodeMetrics>,
}

pub struct RhythmOrchestrator {
    pub policy: RhythmPolicy,
    pub state: Arc<RwLock<RhythmState>>,
}

impl RhythmOrchestrator {
    pub fn new(policy: RhythmPolicy) -> Self {
        Self {
            policy,
            state: Arc::new(RwLock::new(RhythmState::default())),
        }
    }

    pub async fn update_node_metrics(&self, metrics: Vec<NodeMetrics>) {
        let mut state = self.state.write().await;
        state
            .node_metrics
            .extend(metrics.into_iter().map(|m| (m.node_id.clone(), m)));
    }

    pub async fn enqueue_workload(&self, workload: WorkloadDescriptor) {
        let mut state = self.state.write().await;
        state.pending_workloads.push(workload);
    }

    /// Core rhythm decision: assign node + deferral while respecting SLOs.
    pub async fn plan_batch(&self) -> Vec<(RhythmDecision, Option<EnergyOptimizationEvent>)> {
        use chrono::Utc;
        let mut result = Vec::new();
        let mut state = self.state.write().await;

        let metrics_snapshot = state.node_metrics.clone();
        let workloads = std::mem::take(&mut state.pending_workloads);

        for w in workloads {
            let slo = self
                .policy
                .workload_classes
                .get(&w.class)
                .cloned()
                .unwrap_or(WorkloadSlo {
                    max_deferral_ms: 0,
                    target_latency_ms: 500,
                });

            // INTERACTIVE workloads: no deferral allowed.
            let max_deferral = if matches!(w.class, WorkloadClass::INTERACTIVE) {
                0
            } else {
                slo.max_deferral_ms
            };

            // Select best node: lowest power draw and temperature while under 80% utilization.
            let mut selected_node = None;
            let mut best_score = f32::MAX;

            for (node_id, m) in &metrics_snapshot {
                if m.utilization_pct >= 80.0 {
                    continue;
                }
                let score = m.power_draw_watts + (m.temperature_c * 1.5);
                if score < best_score {
                    best_score = score;
                    selected_node = Some(node_id.clone());
                }
            }

            let assigned_node = selected_node.unwrap_or_else(|| "DEFAULT-NODE".to_string());

            // Simple heuristic: half of allowed deferral for non-interactive workloads.
            let deferral_ms = if max_deferral > 0 {
                max_deferral / 2
            } else {
                0
            };

            let decision = RhythmDecision {
                workload_id: w.workload_id.clone(),
                assigned_node_id: assigned_node.clone(),
                deferral_ms,
                batched_with: Vec::new(), // micro-batching extension point
                reason: if deferral_ms > 0 {
                    "ENERGY_SMOOTHING_WITHIN_SLO".into()
                } else {
                    "NO_DEFERRAL_REQUIRED_OR_ALLOWED".into()
                },
            };

            // Estimate energy savings only when deferral is applied.
            let event = if self.policy.logging.enabled && deferral_ms > 0 {
                let est_saved = w.est_energy_joules * 0.1; // placeholder 10% savings
                Some(EnergyOptimizationEvent {
                    event_id: format!("EOE-{}", w.workload_id),
                    workload_id: w.workload_id.clone(),
                    assigned_node_id: assigned_node,
                    deferral_ms,
                    est_energy_saved_joules: est_saved,
                    timestamp_utc: Utc::now().to_rfc3339(),
                })
            } else {
                None
            };

            result.push((decision, event));
        }

        result
    }
}

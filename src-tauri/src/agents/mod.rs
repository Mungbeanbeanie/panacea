//! Agent caste facade: traits and shared lifecycle types.
//!
//! Scouts observe and score; Soldiers wake, remediate, and undergo apoptosis.
//! Cross-module callers reach the agents through this facade, not sibling internals.

pub mod scout;
pub mod soldier;

use crate::core::{BehavioralSchema, Pid, ThreatId};

/// Scout → Soldier wake notification, carrying the `Threat_ID` the Soldier will act on.
///
/// Phase 3 (Soldier lifecycle) consumes this; until then a stub consumer stands in.
#[derive(Debug, Clone)]
pub struct WakeSignal {
    /// Identity of the threat that crossed the Stage-1 threshold.
    pub threat_id: ThreatId,
    /// The suspended process the threat is running as.
    pub pid: Pid,
    /// The flagged behavioral vector itself — the Soldier uses it to pick the mock target
    /// physics for the sandbox (a replicator-style strain resists the standard alleles).
    pub schema: BehavioralSchema,
}

/// Scout → Threat Registry (Ledger 2) submission for a newly flagged trajectory.
///
/// Phase 2 (Threat Registry) consumes this; until then a stub consumer stands in.
#[derive(Debug, Clone)]
pub struct ThreatReport {
    /// Cryptographic identity derived from `schema`.
    pub threat_id: ThreatId,
    /// The behavioral vector that was flagged.
    pub schema: BehavioralSchema,
}

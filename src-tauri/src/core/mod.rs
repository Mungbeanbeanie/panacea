//! Shared config, constants, and cross-cutting types for the core.
//!
//! `ThreatId`, `AnomalyScore`, `Pid`, and `GeneHandle` are the primitives every caste
//! (Scout, Soldier) and ledger (Threat Registry, Genome Registry) share, so a single
//! definition keeps their meaning consistent across module boundaries. See
//! ../../.claude/docs/source-of-truth.md.

use serde::{Deserialize, Serialize};

/// Process ID of the OS process an agent is tracking or has suspended.
pub type Pid = u32;

/// Cumulative Stage 1 behavioral trajectory score for one PID.
///
/// Scouts accumulate this from weighted state-transition actions (Action A +20, B +50,
/// C +40). Crossing [`ANOMALY_THRESHOLD`] fires the hard interrupt that suspends the PID
/// and wakes the local Soldier spore.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct AnomalyScore(pub u32);

/// Cumulative score that fires the Stage 1 hard interrupt (suspend PID, wake Soldier).
pub const ANOMALY_THRESHOLD: AnomalyScore = AnomalyScore(100);

/// Cryptographic hash of a behavioral vector; the primary key of the Threat Registry
/// (Ledger 2). Two Scouts observing the same trajectory produce the same `ThreatId`,
/// which is how the registry correlates sightings into a `Confidence_Score`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ThreatId(pub [u8; 32]);

/// Reference to a compiled Wasm Gene Payload — the `Wasm_Gene_Hash` a Soldier resolves a
/// `ThreatId` to via the Genome Registry (Ledger 3), verifies against the State Ledger's
/// Merkle root, and fetches from IPFS before running in-sandbox.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct GeneHandle(pub [u8; 32]);

/// One anomalous behavioral action observed for a process during Stage 1.
///
/// The PoC vocabulary standing in for real syscall / network / I-O traces; each variant maps
/// to a Source-of-Truth Stage-1 action (scoring weights live in `agents::scout::weight`).
/// Fieldless with explicit discriminants so it folds into the `ThreatId` digest as one stable
/// byte.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Action {
    /// Spawns a hidden child process from a Temp directory (Source of Truth: Action A).
    HiddenChildFromTemp = 0,
    /// Enumerates network adapters while tampering with the Volume Shadow Copy service
    /// (Source of Truth: Action B).
    NetEnumWithVssTamper = 1,
    /// Rapidly loops file handles reading/writing high-entropy data (Source of Truth: Action C).
    HighEntropyFileLoop = 2,
}

/// `Behavioral_Schema` — the ordered action sequence flagged for one process.
///
/// Ledger 2's per-threat behavioral vector (the "sequence of syscalls and network ports").
/// Hashing it yields the [`ThreatId`], so two Scouts observing the same sequence agree on the
/// id without coordinating.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BehavioralSchema {
    /// The flagged actions, in observation order.
    pub actions: Vec<Action>,
}

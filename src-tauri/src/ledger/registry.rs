//! Local cache of the Threat and Genome registries. Genome entries carry Epigenetic_Status
//! (0 active, 1 suppressed); status 1 is the kill-switch that halts a cure
//! (see ../../.claude/skills/suppression-path-test).
//!
//! This file currently holds the Threat Registry (Ledger 2, Phase 2 of
//! ../../.claude/docs/plan.md). The Genome Registry (Phase 6) lands in a section below when
//! that phase starts, per the merge-risk note in ../../.claude/docs/collaboration.md.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::core::ThreatId;

/// Confidence score that triggers a network-wide mobilization command.
///
/// The Source of Truth names the mechanism but not a number (unlike Stage 1's 100-pt
/// threshold). 2 matches plan.md's Phase 2 done-criterion ("two matching trajectories...
/// fire a mobilization") — tune when real global sighting volume is known.
pub const MOBILIZATION_THRESHOLD: u32 = 2;

/// The syscall/network-port sequence Stage 1 flagged, exactly as Scouts will emit it.
///
/// Hashing this (not storing a Scout-assigned ID) is what lets two independent Scouts that
/// witness the same trajectory land on the same [`ThreatId`] and correlate in the registry.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BehavioralSchema(pub Vec<String>);

impl BehavioralSchema {
    /// Cryptographic hash of the vector — the `Threat_ID` primary key for this schema.
    pub fn threat_id(&self) -> ThreatId {
        let mut hasher = Sha256::new();
        for step in &self.0 {
            hasher.update(step.as_bytes());
            hasher.update(b"\0");
        }
        ThreatId(hasher.finalize().into())
    }
}

/// One row of the Threat Registry: a known trajectory and how many independent Scouts
/// globally have reported seeing it.
#[derive(Debug, Clone)]
pub struct ThreatEntry {
    pub schema: BehavioralSchema,
    pub confidence_score: u32,
}

/// Result of correlating one incoming vector against the registry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReportOutcome {
    /// Recorded (new or matched); confidence is still under the mobilization threshold.
    Recorded { confidence_score: u32 },
    /// Confidence just crossed [`MOBILIZATION_THRESHOLD`] — mobilize the network.
    ///
    /// Broadcasting the mobilization command is a Ledger 5 (P2P transport) concern that
    /// doesn't exist yet; returning this variant is the mock for it.
    Mobilized { confidence_score: u32 },
}

/// The Threat Registry (Ledger 2): collective memory of known-bad trajectories.
#[derive(Debug, Default)]
pub struct ThreatRegistry {
    entries: HashMap<ThreatId, ThreatEntry>,
}

impl ThreatRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    /// Correlates an incoming behavioral vector to its `Threat_ID`, inserting a new entry
    /// on first sighting or incrementing `Confidence_Score` on a match, and reports whether
    /// this sighting crossed [`MOBILIZATION_THRESHOLD`].
    pub fn report(&mut self, schema: BehavioralSchema) -> ReportOutcome {
        let threat_id = schema.threat_id();
        let entry = self
            .entries
            .entry(threat_id)
            .or_insert_with(|| ThreatEntry {
                schema,
                confidence_score: 0,
            });
        entry.confidence_score += 1;
        let confidence_score = entry.confidence_score;

        if confidence_score >= MOBILIZATION_THRESHOLD {
            ReportOutcome::Mobilized { confidence_score }
        } else {
            ReportOutcome::Recorded { confidence_score }
        }
    }

    pub fn get(&self, threat_id: &ThreatId) -> Option<&ThreatEntry> {
        self.entries.get(threat_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn vssadmin_trajectory() -> BehavioralSchema {
        BehavioralSchema(vec!["enum_adapters".into(), "vssadmin_modify".into()])
    }

    #[test]
    fn same_trajectory_hashes_to_same_threat_id() {
        assert_eq!(
            vssadmin_trajectory().threat_id(),
            vssadmin_trajectory().threat_id()
        );
    }

    #[test]
    fn different_trajectories_do_not_collide() {
        let other = BehavioralSchema(vec!["temp_spawn".into()]);
        assert_ne!(vssadmin_trajectory().threat_id(), other.threat_id());
    }

    #[test]
    fn second_matching_sighting_mobilizes() {
        let mut registry = ThreatRegistry::new();

        let first = registry.report(vssadmin_trajectory());
        assert_eq!(
            first,
            ReportOutcome::Recorded {
                confidence_score: 1
            }
        );

        let second = registry.report(vssadmin_trajectory());
        assert_eq!(
            second,
            ReportOutcome::Mobilized {
                confidence_score: 2
            }
        );

        let entry = registry.get(&vssadmin_trajectory().threat_id()).unwrap();
        assert_eq!(entry.confidence_score, 2);
    }

    #[test]
    fn unrelated_trajectory_does_not_bump_others_score() {
        let mut registry = ThreatRegistry::new();
        registry.report(vssadmin_trajectory());
        registry.report(BehavioralSchema(vec!["temp_spawn".into()]));

        let entry = registry.get(&vssadmin_trajectory().threat_id()).unwrap();
        assert_eq!(entry.confidence_score, 1);
    }
}

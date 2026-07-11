//! Local cache of the Threat and Genome registries. Genome entries carry Epigenetic_Status
//! (0 active, 1 suppressed); status 1 is the kill-switch that halts a cure
//! (see ../../.claude/skills/suppression-path-test).
//!
//! Threat Registry (Ledger 2, Phase 2) and Genome Registry (Ledger 3, Phase 6) both live
//! here, per the merge-risk note in ../../.claude/docs/collaboration.md.

use std::cell::Cell;
use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::core::{GeneHandle, ThreatId};
use crate::evolution::alleles::GenePayload;

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

/// Kill-switch flag on a Genome Registry row (`Epigenetic_Status`): 0 = active expression,
/// 1 = suppressed. A Soldier must check this *before* fetching or executing the gene (see
/// ../../.claude/skills/suppression-path-test).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EpigeneticStatus {
    Active,
    Suppressed,
}

/// One row of the Genome Registry: the cure for a `Threat_ID`.
#[derive(Debug, Clone)]
pub struct GenomeEntry {
    pub gene_hash: GeneHandle,
    pub ipfs_uri: String,
    pub epigenetic_status: EpigeneticStatus,
}

/// The Genome Registry (Ledger 3): the global pharmacy mapping `Threat_ID → Wasm_Gene_Hash`.
/// Soldiers query it **on demand** by `Threat_ID` — never a passive scan.
#[derive(Debug, Default)]
pub struct GenomeRegistry {
    entries: HashMap<ThreatId, GenomeEntry>,
}

impl GenomeRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    /// Publishes a cure for `threat_id` (the Stage 4 ledger commit — mocked here; the real
    /// consensus gate is Phase 8). Starts `Active`.
    pub fn publish(&mut self, threat_id: ThreatId, gene_hash: GeneHandle, ipfs_uri: String) {
        self.entries.insert(
            threat_id,
            GenomeEntry { gene_hash, ipfs_uri, epigenetic_status: EpigeneticStatus::Active },
        );
    }

    /// On-demand lookup by `Threat_ID` — the only read path a Soldier uses.
    pub fn get(&self, threat_id: &ThreatId) -> Option<&GenomeEntry> {
        self.entries.get(threat_id)
    }

    /// Epigenetic Suppressor Token: flips a gene's status to suppressed so Soldiers stop
    /// executing it. Broadcast mechanics land in Phase 9; this sets the flag it acts on.
    pub fn suppress(&mut self, threat_id: &ThreatId) {
        if let Some(entry) = self.entries.get_mut(threat_id) {
            entry.epigenetic_status = EpigeneticStatus::Suppressed;
        }
    }
}

fn to_hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

/// Mock IPFS: content-addressed store for compiled gene payloads, keyed by the same
/// `Wasm_Gene_Hash` a [`GenomeEntry::ipfs_uri`] points at. Stands in for the real
/// distributed file system (see ../../.claude/docs/security.md).
///
/// Tracks fetch calls so tests can assert a suppressed gene is never fetched (see
/// ../../.claude/skills/suppression-path-test).
#[derive(Debug, Default)]
pub struct MockIpfsStore {
    blobs: HashMap<GeneHandle, GenePayload>,
    fetch_calls: Cell<u32>,
}

impl MockIpfsStore {
    pub fn new() -> Self {
        Self::default()
    }

    /// Stores a compiled gene, returning the `IPFS_URI` to record in the Genome Registry.
    pub fn store(&mut self, gene: GenePayload) -> String {
        let hash = gene.gene_hash();
        self.blobs.insert(hash, gene);
        format!("ipfs://{}", to_hex(&hash.0))
    }

    /// Fetches the bytecode for a verified `Wasm_Gene_Hash`.
    pub fn fetch(&self, gene_hash: &GeneHandle) -> Option<GenePayload> {
        self.fetch_calls.set(self.fetch_calls.get() + 1);
        self.blobs.get(gene_hash).cloned()
    }

    pub fn fetch_calls(&self) -> u32 {
        self.fetch_calls.get()
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

    fn sample_gene() -> GenePayload {
        use crate::evolution::alleles::Allele;
        GenePayload { sequence: vec![Allele::Allele04, Allele::Allele12] }
    }

    #[test]
    fn genome_registry_resolves_a_published_cure_on_demand() {
        let threat_id = vssadmin_trajectory().threat_id();
        let mut ipfs = MockIpfsStore::new();
        let uri = ipfs.store(sample_gene());

        let mut genome = GenomeRegistry::new();
        genome.publish(threat_id, sample_gene().gene_hash(), uri);

        let entry = genome.get(&threat_id).unwrap();
        assert_eq!(entry.epigenetic_status, EpigeneticStatus::Active);
        assert_eq!(ipfs.fetch(&entry.gene_hash).unwrap(), sample_gene());
    }

    #[test]
    fn suppressed_gene_is_flagged_and_never_needs_a_fetch_to_tell() {
        let threat_id = vssadmin_trajectory().threat_id();
        let mut genome = GenomeRegistry::new();
        genome.publish(threat_id, sample_gene().gene_hash(), "ipfs://unused".into());

        genome.suppress(&threat_id);

        let entry = genome.get(&threat_id).unwrap();
        assert_eq!(entry.epigenetic_status, EpigeneticStatus::Suppressed);
    }
}

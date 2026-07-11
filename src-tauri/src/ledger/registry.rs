//! Read-through cache of the on-chain Threat/Genome Registry PDAs (Solana migration
//! Phase 11, box 8) — replaces the in-memory `HashMap`-backed mocks. `MockIpfsStore`
//! moved to `../ipfs.rs` (Q6).

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

pub use bio_digital_defense::{GenomeEntry, ThreatEntry};

use crate::core::ThreatId;
use crate::evolution::alleles::GenePayload;
use crate::ledger::state::SolanaLightClient;
use crate::ledger::LedgerError;

/// The genome-lookup boundary `resolve_and_run` depends on (Phase 11 box 11, Q8) — a real
/// `GenomeRegistry` backs the live path; `FakeGenomeSource` backs the existing offline
/// tests without hitting devnet.
pub trait GenomeSource: Send {
    fn get(&self, threat_id: &ThreatId) -> Result<Option<GenomeEntry>, LedgerError>;
}

/// The gene-publishing boundary the Phase-12 evolve-and-commit flow depends on
/// (`agents::soldier::evolve_and_commit`) — a real `SolanaGeneCommitter` (`../client.rs`)
/// backs the live path; `SharedFakeLedger` backs the offline tests.
pub trait GeneCommitter: Send {
    fn commit_gene(&self, threat_id: &ThreatId, gene: &GenePayload) -> Result<(), LedgerError>;
}

/// Read-through wrapper over the Genome Registry PDA — "cache" means "the read path," not
/// a persistent local store (plan.md box 8 wording). Deliberately **no** memoization: this
/// is reused across every wake signal in the long-running `Soldier::run` loop, and
/// `source-of-truth.md` requires a suppression to take effect on the *next* RPC query — a
/// cached read could silently mask a suppression that landed between two dispenses of the
/// same `Threat_ID` (caught by `suppression-path-test` during this review).
pub struct GenomeRegistry {
    light_client: SolanaLightClient,
}

impl GenomeRegistry {
    pub fn new(light_client: SolanaLightClient) -> Self {
        Self { light_client }
    }
}

impl GenomeSource for GenomeRegistry {
    /// On-demand lookup by `Threat_ID` — never a passive scan, mirrors
    /// `GenomeRegistry::get()` in the old mock. Always a fresh RPC read.
    fn get(&self, threat_id: &ThreatId) -> Result<Option<GenomeEntry>, LedgerError> {
        self.light_client.get_genome_entry(threat_id)
    }
}

/// Read-through wrapper over the Threat Registry PDA — mirrors `ThreatRegistry::get()` in
/// the old mock. Writes go through `ledger::client::SolanaConjugationLink::submit_threat`.
pub struct ThreatRegistry {
    light_client: SolanaLightClient,
}

impl ThreatRegistry {
    pub fn new(light_client: SolanaLightClient) -> Self {
        Self { light_client }
    }

    pub fn get(&self, threat_id: &ThreatId) -> Result<Option<ThreatEntry>, LedgerError> {
        self.light_client.get_threat_entry(threat_id)
    }
}

/// Offline test double for `GenomeSource` — an in-memory `HashMap`, so
/// `pharmacy_flow_tests`/`live_dispense_tests` keep running fast without devnet (Q8).
#[derive(Debug, Default)]
pub struct FakeGenomeSource {
    entries: HashMap<ThreatId, GenomeEntry>,
}

impl FakeGenomeSource {
    pub fn new() -> Self {
        Self::default()
    }

    /// Publishes a cure for `threat_id`, starting `Active` — mirrors
    /// `GenomeRegistry::publish()` in the old mock.
    pub fn publish(&mut self, threat_id: ThreatId, gene: &GenePayload) {
        self.entries.insert(
            threat_id,
            GenomeEntry {
                threat_id: threat_id.0,
                gene_hash: gene.gene_hash().0,
                gene_seq: gene.to_bytes(),
                epigenetic_status: 0,
                bump: 0,
            },
        );
    }

    /// Flips a gene's status to suppressed — mirrors `GenomeRegistry::suppress()`.
    pub fn suppress(&mut self, threat_id: &ThreatId) {
        if let Some(entry) = self.entries.get_mut(threat_id) {
            entry.epigenetic_status = 1;
        }
    }

    /// Inserts a `gene_hash`/`gene_seq` pair without requiring they actually match —
    /// `publish()` always computes a consistent pair, so tests that need to model corrupt
    /// on-chain data (mismatched hash) go through this instead.
    pub fn publish_raw(&mut self, threat_id: ThreatId, gene_hash: [u8; 32], gene_seq: Vec<u8>) {
        self.entries.insert(
            threat_id,
            GenomeEntry {
                threat_id: threat_id.0,
                gene_hash,
                gene_seq,
                epigenetic_status: 0,
                bump: 0,
            },
        );
    }
}

impl GenomeSource for FakeGenomeSource {
    fn get(&self, threat_id: &ThreatId) -> Result<Option<GenomeEntry>, LedgerError> {
        Ok(self.entries.get(threat_id).cloned())
    }
}

/// Offline test double spanning both pharmacy boundaries at once: implements both
/// `GenomeSource` and `GeneCommitter` over one shared `FakeGenomeSource`, so a test can
/// exercise the full evolve → commit → re-read loop (`agents::soldier::evolve_and_commit`)
/// without devnet — a commit through one handle is immediately visible through the other,
/// the way a real commit_gene transaction becomes visible to the next RPC read.
#[derive(Clone, Default)]
pub struct SharedFakeLedger(Arc<Mutex<FakeGenomeSource>>);

impl SharedFakeLedger {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn suppress(&self, threat_id: &ThreatId) {
        self.0.lock().unwrap().suppress(threat_id);
    }
}

impl GenomeSource for SharedFakeLedger {
    fn get(&self, threat_id: &ThreatId) -> Result<Option<GenomeEntry>, LedgerError> {
        self.0.lock().unwrap().get(threat_id)
    }
}

impl GeneCommitter for SharedFakeLedger {
    fn commit_gene(&self, threat_id: &ThreatId, gene: &GenePayload) -> Result<(), LedgerError> {
        self.0.lock().unwrap().publish(*threat_id, gene);
        Ok(())
    }
}

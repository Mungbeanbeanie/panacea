//! Read-through cache of the on-chain Threat/Genome Registry PDAs (Solana migration
//! Phase 11, box 8) — replaces the in-memory `HashMap`-backed mocks. `MockIpfsStore`
//! moved to `../ipfs.rs` (Q6).

use std::cell::RefCell;
use std::collections::HashMap;

pub use bio_digital_defense::{GenomeEntry, ThreatEntry};

use crate::core::ThreatId;
use crate::ledger::state::SolanaLightClient;
use crate::ledger::LedgerError;

/// The genome-lookup boundary `resolve_and_run` depends on (Phase 11 box 11, Q8) — a real
/// `GenomeRegistry` backs the live path; `FakeGenomeSource` backs the existing offline
/// tests without hitting devnet.
pub trait GenomeSource {
    fn get(&self, threat_id: &ThreatId) -> Result<Option<GenomeEntry>, LedgerError>;
}

/// Read-through wrapper over the Genome Registry PDA — "cache" means "the read path," not
/// a persistent local store (plan.md box 8 wording). A short-lived in-memory memo avoids a
/// second RPC round-trip for the same `Threat_ID` within one Soldier dispense.
pub struct GenomeRegistry {
    light_client: SolanaLightClient,
    memo: RefCell<HashMap<ThreatId, Option<GenomeEntry>>>,
}

impl GenomeRegistry {
    pub fn new(light_client: SolanaLightClient) -> Self {
        Self {
            light_client,
            memo: RefCell::new(HashMap::new()),
        }
    }
}

impl GenomeSource for GenomeRegistry {
    /// On-demand lookup by `Threat_ID` — never a passive scan, mirrors
    /// `GenomeRegistry::get()` in the old mock.
    fn get(&self, threat_id: &ThreatId) -> Result<Option<GenomeEntry>, LedgerError> {
        if let Some(cached) = self.memo.borrow().get(threat_id) {
            return Ok(cached.clone());
        }
        let entry = self.light_client.get_genome_entry(threat_id)?;
        self.memo.borrow_mut().insert(*threat_id, entry.clone());
        Ok(entry)
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
    pub fn publish(&mut self, threat_id: ThreatId, gene_hash: [u8; 32], ipfs_cid: String) {
        self.entries.insert(
            threat_id,
            GenomeEntry {
                threat_id: threat_id.0,
                gene_hash,
                ipfs_cid,
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
}

impl GenomeSource for FakeGenomeSource {
    fn get(&self, threat_id: &ThreatId) -> Result<Option<GenomeEntry>, LedgerError> {
        Ok(self.entries.get(threat_id).cloned())
    }
}

use anchor_lang::prelude::*;

use crate::constants::GENE_SEQ_MAX_LEN;

/// Threat Registry (Ledger 2) row: one known-bad behavioral trajectory and how many
/// independent Scouts globally have reported it. Mirrors `ledger::registry::ThreatEntry`
/// in the Rust core mock; `threat_id` is both the row key and the PDA seed.
#[account]
#[derive(InitSpace, Debug)]
pub struct ThreatEntry {
    pub threat_id: [u8; 32],
    /// Hash of the `Behavioral_Schema` (the syscall/network-port sequence), not the raw
    /// sequence itself — keeps the account fixed-size; the schema stays off-chain.
    pub behavioral_schema_hash: [u8; 32],
    pub confidence_score: u32,
    pub bump: u8,
}

/// Genome Registry (Ledger 3) row: the cure for a `Threat_ID`. Mirrors
/// `ledger::registry::GenomeEntry`; `epigenetic_status` 0 = Active, 1 = Suppressed (see
/// suppression-path-test — the kill-switch check reads this field before any gene
/// fetch/exec). `gene_seq` is the compiled allele sequence itself — a handful of bytes,
/// smaller than an off-chain content address would be, so the account is the gene's only
/// home (no IPFS/CID indirection). `gene_hash` stays as the documented `Wasm_Gene_Hash`
/// identity, verified by the Soldier against a hash it recomputes from `gene_seq`.
#[account]
#[derive(InitSpace, Debug)]
pub struct GenomeEntry {
    pub threat_id: [u8; 32],
    pub gene_hash: [u8; 32],
    #[max_len(GENE_SEQ_MAX_LEN)]
    pub gene_seq: Vec<u8>,
    pub epigenetic_status: u8,
    pub bump: u8,
}

//! Ledger facade: the unified interface the rest of the core uses to reach the State
//! Ledger, Threat Registry, and Genome Registry. See ../../.claude/docs/architecture.md.
//!
//! `consensus.rs` is the pre-Solana-migration mock of Stage 4 (`ImmunityProof`,
//! `run_poi_consensus`) — superseded by the real on-chain `commit_gene` instruction
//! (`programs/bio_digital_defense`) once `client.rs`/`state.rs`/`registry.rs` wire to it.
//! Left in place (unused) rather than deleted here, since deleting it wasn't part of this
//! change's scope.

pub mod client;
pub mod consensus;
pub mod ipfs;
pub mod registry;
pub mod state;

/// Errors from talking to the live Solana devnet program or the IPFS pinning service —
/// none of these were possible against the in-memory mocks they replace.
#[derive(Debug, thiserror::Error)]
pub enum LedgerError {
    #[error("Solana RPC/client error: {0}")]
    Solana(#[from] anchor_client::ClientError),
    #[error("failed to deserialize on-chain account: {0}")]
    AccountDecode(#[from] anchor_client::anchor_lang::error::Error),
    #[error("PINATA_JWT environment variable is not set")]
    MissingPinataJwt,
    #[error("IPFS pinning-service HTTP error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("fetched gene's hash does not match the on-chain Wasm_Gene_Hash")]
    GeneHashMismatch,
    #[error("no gene payload found for the given CID")]
    GeneNotFound,
}

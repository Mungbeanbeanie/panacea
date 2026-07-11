//! Ledger facade: the unified interface the rest of the core uses to reach the State
//! Ledger, Threat Registry, and Genome Registry. See ../../.claude/docs/architecture.md.

pub mod client;
pub mod registry;
pub mod state;

/// Errors from talking to the live Solana devnet program — none of these were possible
/// against the in-memory mocks they replace.
#[derive(Debug, thiserror::Error)]
pub enum LedgerError {
    #[error("Solana RPC/client error: {0}")]
    Solana(#[from] anchor_client::ClientError),
    #[error("Solana RPC error: {0}")]
    Rpc(#[from] anchor_client::SolanaClientError),
    #[error("failed to deserialize on-chain account: {0}")]
    AccountDecode(#[from] anchor_client::anchor_lang::error::Error),
    #[error("on-chain gene_seq does not match the recorded Wasm_Gene_Hash")]
    GeneHashMismatch,
}

//! Threat Registry (Ledger 2) and Genome Registry (Ledger 3) as real Solana devnet program
//! accounts. See ../../../.claude/docs/source-of-truth.md.

pub mod constants;
pub mod error;
pub mod instructions;
pub mod state;

use anchor_lang::prelude::*;

pub use constants::*;
pub use instructions::*;
pub use state::*;

declare_id!("5r6fERX6CtJ8RHzpf5wLtY1SfptV8rk4JEHfu6yh5WZV");

#[program]
pub mod bio_digital_defense {
    use super::*;

    /// Threat Registry (Ledger 2): correlates a Scout's behavioral vector to its
    /// `Threat_ID`, creating or incrementing the row's `Confidence_Score`.
    pub fn submit_threat(
        ctx: Context<SubmitThreat>,
        threat_id: [u8; 32],
        behavioral_schema_hash: [u8; 32],
    ) -> Result<()> {
        crate::instructions::submit_threat::handle_submit_threat(
            ctx,
            threat_id,
            behavioral_schema_hash,
        )
    }

    /// Genome Registry (Ledger 3): publishes a cure for `threat_id`, gated by the 3-of-5
    /// Lymph Node multisig (Proof of Immunity).
    pub fn commit_gene(
        ctx: Context<CommitGene>,
        threat_id: [u8; 32],
        gene_hash: [u8; 32],
        gene_seq: Vec<u8>,
    ) -> Result<()> {
        crate::instructions::commit_gene::handle_commit_gene(ctx, threat_id, gene_hash, gene_seq)
    }

    /// Epigenetic Suppressor Token: flips `Epigenetic_Status` to 1 on an existing Genome
    /// Registry PDA, gated by the same 3-of-5 Lymph Node multisig as `commit_gene`.
    pub fn suppress_gene(ctx: Context<SuppressGene>, threat_id: [u8; 32]) -> Result<()> {
        crate::instructions::suppress_gene::handle_suppress_gene(ctx, threat_id)
    }
}

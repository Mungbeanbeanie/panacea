use anchor_lang::prelude::*;

use crate::constants::*;
use crate::error::ErrorCode;
use crate::state::GenomeEntry;

#[derive(Accounts)]
#[instruction(threat_id: [u8; 32])]
pub struct SuppressGene<'info> {
    // Not `init_if_needed` — suppressing a gene that was never committed is meaningless.
    // If the PDA doesn't exist, Anchor's deserialization fails with `AccountNotInitialized`.
    #[account(
        mut,
        seeds = [GENOME_SEED, threat_id.as_ref()],
        bump = genome_entry.bump,
    )]
    pub genome_entry: Account<'info, GenomeEntry>,

    /// Same 5 hardcoded `LYMPH_NODE_VALIDATORS` / `POI_QUORUM` multisig as `commit_gene` —
    /// the Source of Truth's "same multisig authority" for the kill-switch.
    pub validator_1: Option<Signer<'info>>,
    pub validator_2: Option<Signer<'info>>,
    pub validator_3: Option<Signer<'info>>,
    pub validator_4: Option<Signer<'info>>,
    pub validator_5: Option<Signer<'info>>,
}

/// Epigenetic Suppressor Token: flips `epigenetic_status` to `1` (Suppressed) on an
/// existing Genome Registry PDA, gated by the same 3-of-5 Lymph Node multisig as
/// `commit_gene`. The on-chain replacement for `GenomeRegistry::suppress()` +
/// `MockConjugationLink::broadcast_suppressor()` combined into one transaction — a Soldier
/// checking this account on its next RPC query sees the flag instantly, no separate
/// broadcast/drain step needed.
pub fn handle_suppress_gene(ctx: Context<SuppressGene>, _threat_id: [u8; 32]) -> Result<()> {
    let candidates = [
        ctx.accounts.validator_1.as_ref(),
        ctx.accounts.validator_2.as_ref(),
        ctx.accounts.validator_3.as_ref(),
        ctx.accounts.validator_4.as_ref(),
        ctx.accounts.validator_5.as_ref(),
    ];

    let signed_count = LYMPH_NODE_VALIDATORS
        .iter()
        .filter(|known_key| {
            candidates
                .iter()
                .flatten()
                .any(|signer| signer.key() == **known_key)
        })
        .count() as u8;

    require!(signed_count >= POI_QUORUM, ErrorCode::InsufficientQuorum);

    ctx.accounts.genome_entry.epigenetic_status = 1;

    Ok(())
}

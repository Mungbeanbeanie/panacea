use anchor_lang::prelude::*;

use crate::constants::*;
use crate::error::ErrorCode;
use crate::state::GenomeEntry;

#[derive(Accounts)]
#[instruction(threat_id: [u8; 32])]
pub struct CommitGene<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(
        init_if_needed,
        payer = payer,
        space = 8 + GenomeEntry::INIT_SPACE,
        seeds = [GENOME_SEED, threat_id.as_ref()],
        bump,
    )]
    pub genome_entry: Account<'info, GenomeEntry>,

    pub system_program: Program<'info, System>,

    /// One of the 5 hardcoded `LYMPH_NODE_VALIDATORS` — optional because a given
    /// transaction only needs to carry `POI_QUORUM` (3) of the 5 as actual signers, not
    /// all 5. The client omits a slot by passing this program's own ID as the account.
    pub validator_1: Option<Signer<'info>>,
    pub validator_2: Option<Signer<'info>>,
    pub validator_3: Option<Signer<'info>>,
    pub validator_4: Option<Signer<'info>>,
    pub validator_5: Option<Signer<'info>>,
}

/// Publishes a cure for `threat_id`, gated by a 3-of-5 Lymph Node multisig (Proof of
/// Immunity) — the on-chain replacement for `consensus::run_poi_consensus()` +
/// `consensus::commit_gene()` + `GenomeRegistry::publish()` combined. Starts `Active`
/// (`epigenetic_status = 0`); flipping to `1` is `suppress_gene`. Refuses to overwrite an
/// already-suppressed entry back to `Active`.
pub fn handle_commit_gene(
    ctx: Context<CommitGene>,
    threat_id: [u8; 32],
    gene_hash: [u8; 32],
    gene_seq: Vec<u8>,
) -> Result<()> {
    require!(
        gene_seq.len() <= GENE_SEQ_MAX_LEN,
        ErrorCode::GeneSequenceTooLong
    );

    let candidates = [
        ctx.accounts.validator_1.as_ref(),
        ctx.accounts.validator_2.as_ref(),
        ctx.accounts.validator_3.as_ref(),
        ctx.accounts.validator_4.as_ref(),
        ctx.accounts.validator_5.as_ref(),
    ];

    // Count distinct *known* validators represented among the provided signers, not the
    // number of filled slots — passing the same real validator in multiple slots can't
    // inflate the count above 1 for that validator.
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

    let entry = &mut ctx.accounts.genome_entry;

    // The Source of Truth has no reactivation path once a gene is suppressed, so a fresh
    // commit_gene must not silently undo it (init_if_needed would otherwise happily
    // overwrite epigenetic_status back to Active).
    require!(entry.epigenetic_status != 1, ErrorCode::GeneSuppressed);

    entry.threat_id = threat_id;
    entry.gene_hash = gene_hash;
    entry.gene_seq = gene_seq;
    entry.epigenetic_status = 0;
    entry.bump = ctx.bumps.genome_entry;

    Ok(())
}

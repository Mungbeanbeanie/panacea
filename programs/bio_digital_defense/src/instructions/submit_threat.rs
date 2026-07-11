use anchor_lang::prelude::*;

use crate::constants::*;
use crate::state::ThreatEntry;

#[derive(Accounts)]
#[instruction(threat_id: [u8; 32])]
pub struct SubmitThreat<'info> {
    /// Any Scout-bearing endpoint may report a sighting — no gate here, matching
    /// `ThreatRegistry::report()` being open to any caller in the Rust core mock.
    #[account(mut)]
    pub reporter: Signer<'info>,

    #[account(
        init_if_needed,
        payer = reporter,
        space = 8 + ThreatEntry::INIT_SPACE,
        seeds = [THREAT_SEED, threat_id.as_ref()],
        bump,
    )]
    pub threat_entry: Account<'info, ThreatEntry>,

    pub system_program: Program<'info, System>,
}

/// Emitted the moment a threat's `confidence_score` crosses `MOBILIZATION_THRESHOLD` — the
/// on-chain substitute for the mock's `ReportOutcome::Mobilized` return value, since an
/// instruction has no caller to hand a Rust enum back to. Fires once, at the crossing,
/// rather than on every subsequent repeat sighting.
#[event]
pub struct ThreatMobilized {
    pub threat_id: [u8; 32],
    pub confidence_score: u32,
}

/// Correlates an incoming behavioral vector to its `Threat_ID`: creates the PDA on first
/// sighting (`confidence_score = 1`) or increments it on a repeat sighting, mirroring
/// `ledger::registry::ThreatRegistry::report()`.
pub fn handle_submit_threat(
    ctx: Context<SubmitThreat>,
    threat_id: [u8; 32],
    behavioral_schema_hash: [u8; 32],
) -> Result<()> {
    let entry = &mut ctx.accounts.threat_entry;

    // A freshly created PDA's account data is zero-allocated, so `confidence_score == 0`
    // reliably means "this call created the row" — no separate initialized flag needed.
    if entry.confidence_score == 0 {
        entry.threat_id = threat_id;
        entry.behavioral_schema_hash = behavioral_schema_hash;
        entry.bump = ctx.bumps.threat_entry;
    }
    entry.confidence_score += 1;

    if entry.confidence_score == MOBILIZATION_THRESHOLD {
        emit!(ThreatMobilized {
            threat_id,
            confidence_score: entry.confidence_score,
        });
    }

    Ok(())
}

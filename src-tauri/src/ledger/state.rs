//! Solana light-client reads: commitment-level account fetches, replacing the custom
//! Merkle-path scheme (Phase 11 box 7 — see ../../.claude/docs/source-of-truth.md, Ledger 1
//! and Ledger 2/3). Reading a `confirmed`-commitment account already carries Solana's own
//! integrity guarantee (Q3: `confirmed` chosen over `finalized` for demo responsiveness;
//! the Source of Truth permits either), so there is no separate proof object to construct
//! or verify.

use std::sync::Arc;

use anchor_client::anchor_lang::prelude::Pubkey;
use anchor_client::anchor_lang::AccountDeserialize;
use anchor_client::{Client, Cluster, CommitmentConfig};
use solana_keypair::Keypair;
use solana_rpc_client::rpc_client::RpcClient;

use bio_digital_defense::{GenomeEntry, ThreatEntry};

use crate::core::ThreatId;
use crate::ledger::LedgerError;

/// PDA seed prefixes, mirrored by hand from
/// `programs/bio_digital_defense/src/constants.rs` — the client and on-chain program are
/// separate crates, so these stay in sync manually.
const THREAT_SEED: &[u8] = b"threat";
const GENOME_SEED: &[u8] = b"genome";

pub fn threat_pda(threat_id: &ThreatId) -> Pubkey {
    Pubkey::find_program_address(&[THREAT_SEED, &threat_id.0], &bio_digital_defense::ID).0
}

pub fn genome_pda(threat_id: &ThreatId) -> Pubkey {
    Pubkey::find_program_address(&[GENOME_SEED, &threat_id.0], &bio_digital_defense::ID).0
}

/// Solana devnet RPC light client — an endpoint downloads no chain, just queries account
/// state directly at `confirmed` commitment. Built via `anchor_client` (rather than a
/// hand-constructed RPC client) so its account-fetch is guaranteed version-compatible with
/// the anchor-lang types (`ThreatEntry`/`GenomeEntry`) it deserializes into.
pub struct SolanaLightClient {
    rpc: RpcClient,
}

impl SolanaLightClient {
    /// `payer` isn't used for signing here (reads need no signature) — a `Program` handle
    /// still requires one to construct, so this reuses whichever keypair the caller has on
    /// hand (Q5: the single demo node's deployer wallet).
    pub fn new(cluster: Cluster, payer: Arc<Keypair>) -> Result<Self, LedgerError> {
        let client = Client::new_with_options(cluster, payer, CommitmentConfig::confirmed());
        let program = client.program(bio_digital_defense::ID)?;
        Ok(Self { rpc: program.rpc() })
    }

    fn get_account<T: AccountDeserialize>(
        &self,
        address: &Pubkey,
    ) -> Result<Option<T>, LedgerError> {
        match self
            .rpc
            .get_account_with_commitment(address, CommitmentConfig::confirmed())?
            .value
        {
            Some(account) => {
                let mut data: &[u8] = &account.data;
                Ok(Some(T::try_deserialize(&mut data)?))
            }
            None => Ok(None),
        }
    }

    /// On-demand lookup by `Threat_ID` — mirrors `ThreatRegistry::get()` in the old mock.
    pub fn get_threat_entry(&self, threat_id: &ThreatId) -> Result<Option<ThreatEntry>, LedgerError> {
        self.get_account(&threat_pda(threat_id))
    }

    /// On-demand lookup by `Threat_ID` — the only read path a Soldier uses (never a
    /// passive scan), mirrors `GenomeRegistry::get()` in the old mock.
    pub fn get_genome_entry(&self, threat_id: &ThreatId) -> Result<Option<GenomeEntry>, LedgerError> {
        self.get_account(&genome_pda(threat_id))
    }
}

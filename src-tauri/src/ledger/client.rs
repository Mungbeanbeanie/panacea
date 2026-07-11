//! Real Solana RPC transport ("conjugation") for the on-chain Threat/Genome Registry
//! (Solana migration Phase 11, box 6) — replaces `MockConjugationLink`'s P2P stand-in.
//! Every write here is a signed Solana transaction; confirmation *is* the commit, so there
//! is no separate block to assemble.

use std::sync::Arc;

use anchor_client::anchor_lang::solana_program::system_program;
use anchor_client::{Client, Cluster, Program, Signer};
use bio_digital_defense::{accounts, instruction};
use solana_keypair::Keypair;
use solana_signature::Signature;

use crate::core::{GeneHandle, ThreatId};
use crate::ledger::ipfs::Cid;
use crate::ledger::state::{genome_pda, threat_pda};
use crate::ledger::LedgerError;

/// Real Solana RPC transport, replacing `MockConjugationLink`. Holds one
/// `anchor_client::Program` handle (payer + cluster), built once and reused for every
/// instruction this endpoint submits.
pub struct SolanaConjugationLink {
    program: Program<Arc<Keypair>>,
}

impl SolanaConjugationLink {
    pub fn new(cluster: Cluster, payer: Arc<Keypair>) -> Result<Self, LedgerError> {
        let client = Client::new(cluster, payer);
        let program = client.program(bio_digital_defense::ID)?;
        Ok(Self { program })
    }

    /// Threat Registry (Ledger 2): create/update the `Threat_ID`'s PDA, incrementing
    /// `Confidence_Score` — mirrors `ThreatRegistry::report()`. Any funded devnet keypair
    /// can call this (the Scout's own signer — Q5: the same deployer wallet).
    pub fn submit_threat(
        &self,
        threat_id: ThreatId,
        behavioral_schema_hash: [u8; 32],
    ) -> Result<Signature, LedgerError> {
        let threat_entry = threat_pda(&threat_id);
        let signature = self
            .program
            .request()
            .accounts(accounts::SubmitThreat {
                reporter: self.program.payer(),
                threat_entry,
                system_program: system_program::ID,
            })
            .args(instruction::SubmitThreat {
                threat_id: threat_id.0,
                behavioral_schema_hash,
            })
            .send()?;
        Ok(signature)
    }

    /// Genome Registry (Ledger 3): publish a cure, gated by the 3-of-5 Lymph Node multisig
    /// (Proof of Immunity) — mirrors `consensus::commit_gene()` + `GenomeRegistry::publish()`
    /// combined. `validators` holds the Lymph Node keypair files (Q4: one process co-signs
    /// on behalf of all 5).
    pub fn commit_gene(
        &self,
        threat_id: ThreatId,
        gene_hash: GeneHandle,
        ipfs_cid: Cid,
        validators: &[Keypair],
    ) -> Result<Signature, LedgerError> {
        let genome_entry = genome_pda(&threat_id);
        let mut request = self
            .program
            .request()
            .accounts(accounts::CommitGene {
                payer: self.program.payer(),
                genome_entry,
                system_program: system_program::ID,
                validator_1: validators.first().map(Signer::pubkey),
                validator_2: validators.get(1).map(Signer::pubkey),
                validator_3: validators.get(2).map(Signer::pubkey),
                validator_4: validators.get(3).map(Signer::pubkey),
                validator_5: validators.get(4).map(Signer::pubkey),
            })
            .args(instruction::CommitGene {
                threat_id: threat_id.0,
                gene_hash: gene_hash.0,
                ipfs_cid: ipfs_cid.0,
            });
        for validator in validators {
            request = request.signer(validator);
        }
        Ok(request.send()?)
    }

    /// Epigenetic Suppressor Token: flips `Epigenetic_Status` to 1 on an existing Genome
    /// Registry PDA, gated by the same multisig as `commit_gene` — mirrors
    /// `GenomeRegistry::suppress()` and the old mock's broadcast/drain pair, collapsed into
    /// one transaction (no separate "broadcast then drain" step once suppression *is* the
    /// transaction itself).
    pub fn suppress_gene(
        &self,
        threat_id: ThreatId,
        validators: &[Keypair],
    ) -> Result<Signature, LedgerError> {
        let genome_entry = genome_pda(&threat_id);
        let mut request = self
            .program
            .request()
            .accounts(accounts::SuppressGene {
                genome_entry,
                validator_1: validators.first().map(Signer::pubkey),
                validator_2: validators.get(1).map(Signer::pubkey),
                validator_3: validators.get(2).map(Signer::pubkey),
                validator_4: validators.get(3).map(Signer::pubkey),
                validator_5: validators.get(4).map(Signer::pubkey),
            })
            .args(instruction::SuppressGene {
                threat_id: threat_id.0,
            });
        for validator in validators {
            request = request.signer(validator);
        }
        Ok(request.send()?)
    }
}

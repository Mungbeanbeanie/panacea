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

use crate::core::ThreatId;
use crate::evolution::alleles::CompiledGene;
use crate::ledger::registry::GeneCommitter;
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
    /// on behalf of all 5). Stores `gene`'s compiled Wasm bytes directly in the account (no
    /// IPFS/CID indirection — see `../../.claude/docs/source-of-truth.md`, Ledger 3).
    pub fn commit_gene(
        &self,
        threat_id: ThreatId,
        gene: &CompiledGene,
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
                gene_hash: gene.gene_hash().0,
                gene_seq: gene.to_bytes(),
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

/// The `Pharmacy`'s write side for the Phase-12 evolve-and-commit flow
/// (`agents::soldier::evolve_and_commit`): a `SolanaConjugationLink` plus the Lymph Node
/// validator keypairs needed to co-sign `commit_gene`/`suppress_gene`. Held behind `Arc` so
/// `spawn_demo` can hand a `GeneCommitter` trait object to the `Pharmacy` while keeping its
/// own handle to call `suppress` directly once the happy-path wave is done.
pub struct SolanaGeneCommitter {
    link: SolanaConjugationLink,
    validators: Vec<Keypair>,
}

impl SolanaGeneCommitter {
    pub fn new(link: SolanaConjugationLink, validators: Vec<Keypair>) -> Self {
        Self { link, validators }
    }

    /// The kill-switch: flips `Epigenetic_Status` to 1 for `threat_id`'s gene.
    pub fn suppress(&self, threat_id: ThreatId) -> Result<Signature, LedgerError> {
        self.link.suppress_gene(threat_id, &self.validators)
    }
}

impl GeneCommitter for Arc<SolanaGeneCommitter> {
    fn commit_gene(&self, threat_id: &ThreatId, gene: &CompiledGene) -> Result<(), LedgerError> {
        self.link
            .commit_gene(*threat_id, gene, &self.validators)
            .map(|_signature| ())
    }
}

#[cfg(test)]
mod live_devnet_tests {
    use solana_keypair::read_keypair_file;

    use super::*;
    use crate::ledger::registry::GenomeSource;
    use crate::ledger::state::SolanaLightClient;

    fn deployer() -> Arc<Keypair> {
        let path = format!(
            "{}/.config/solana/id.json",
            std::env::var("HOME").expect("HOME not set")
        );
        Arc::new(read_keypair_file(&path).expect("read deployer keypair"))
    }

    fn validators() -> Vec<Keypair> {
        let manifest_dir = env!("CARGO_MANIFEST_DIR");
        (1..=5)
            .map(|i| {
                let path = format!("{manifest_dir}/../keys/lymph-nodes/validator-{i}.json");
                read_keypair_file(&path).expect("read Lymph Node validator keypair")
            })
            .collect()
    }

    /// Exercises the real on-chain path end to end against Solana devnet: `submit_threat`,
    /// then a 3-of-5 multisig `commit_gene` + `suppress_gene`, reading each result back via
    /// `SolanaLightClient`/`ThreatRegistry`/`GenomeRegistry`. Costs real devnet SOL (tiny) and
    /// takes several seconds for transaction confirmation, so it's `#[ignore]`d like the
    /// existing real-process tests — run locally with `cargo test -- --ignored`.
    #[test]
    #[ignore = "hits live Solana devnet; run locally with --ignored"]
    fn submit_commit_and_suppress_round_trip_against_devnet() {
        use crate::ledger::registry::{GenomeRegistry, ThreatRegistry};

        let payer = deployer();
        let validators = validators();
        let conjugation =
            SolanaConjugationLink::new(Cluster::Devnet, payer.clone()).expect("connect");

        // A fresh threat_id each run so repeated test runs don't collide with a
        // previously-suppressed PDA from an earlier pass.
        let threat_id = ThreatId(rand_bytes());

        conjugation
            .submit_threat(threat_id, [0xAA; 32])
            .expect("submit_threat");

        let threat_registry =
            ThreatRegistry::new(SolanaLightClient::new(Cluster::Devnet, payer.clone()).expect("connect"));
        let threat_entry = threat_registry
            .get(&threat_id)
            .expect("read threat entry")
            .expect("threat entry exists after submit_threat");
        assert_eq!(threat_entry.confidence_score, 1);

        let gene = crate::evolution::alleles::GenePayload {
            sequence: vec![
                crate::evolution::alleles::Allele::Allele04,
                crate::evolution::alleles::Allele::Allele12,
            ],
        };
        let compiled = gene.compile();
        conjugation
            .commit_gene(threat_id, &compiled, &validators)
            .expect("commit_gene");

        let genome_registry =
            GenomeRegistry::new(SolanaLightClient::new(Cluster::Devnet, payer.clone()).expect("connect"));
        let genome_entry = genome_registry
            .get(&threat_id)
            .expect("read genome entry")
            .expect("genome entry exists after commit_gene");
        assert_eq!(genome_entry.epigenetic_status, 0, "starts Active");
        assert_eq!(genome_entry.gene_hash, compiled.gene_hash().0);
        assert_eq!(genome_entry.gene_seq, compiled.to_bytes());

        conjugation
            .suppress_gene(threat_id, &validators)
            .expect("suppress_gene");

        // Deliberately reusing the *same* `genome_registry` instance from the pre-suppression
        // read above — this is the regression test for a caching bug this review caught and
        // fixed: `GenomeRegistry` used to memoize reads per `Threat_ID`, which would have
        // silently returned the stale `Active` entry here instead of re-querying the chain.
        // `source-of-truth.md` requires a suppression to take effect on the Soldier's *next*
        // RPC query, so `GenomeRegistry` now has no memo at all — every `get()` is fresh.
        let entry_after = genome_registry
            .get(&threat_id)
            .expect("read genome entry")
            .expect("still exists");
        assert_eq!(entry_after.epigenetic_status, 1, "suppress_gene flips it to Suppressed");
    }

    fn rand_bytes() -> [u8; 32] {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .subsec_nanos();
        let mut bytes = [0u8; 32];
        bytes[..4].copy_from_slice(&nanos.to_le_bytes());
        bytes
    }
}

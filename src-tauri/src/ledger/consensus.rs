//! Stage 4 — Consensus & Ledger Commitment.
//!
//! A candidate gene that passed Stage 3 (../../evolution/lymph_node.rs) gets a
//! Zero-Knowledge Proof (mock) that it neutralizes the threat and is non-autoimmune — the
//! proof carries only a commitment hash of that claim, never the sandbox trace or malware
//! payload it was derived from. Network validators verify it via Proof-of-Immunity
//! consensus (mock); once quorum is reached, [`commit_gene`] publishes the row to the
//! Genome Registry and folds it into a new State Ledger block. A proof that never reached
//! quorum has no path to [`commit_gene`] — see ../../.claude/docs/plan.md, Phase 8.

use sha2::{Digest, Sha256};

use crate::core::{GeneHandle, ThreatId};
use crate::evolution::alleles::GenePayload;
use crate::evolution::lymph_node::RegressionOutcome;
use crate::evolution::sandbox::TrialOutcome;
use crate::ledger::client::MockConjugationLink;
use crate::ledger::registry::GenomeRegistry;
use crate::ledger::state::{BlockHeader, Hash};

/// Zero-Knowledge Proof (mock) that a gene neutralizes `threat_id` and passed the Stage 3
/// allergy check. A validator re-derives the same commitment and compares, rather than
/// replaying the underlying sandbox/regression evidence — that's what keeps the raw
/// host/malware details out of the proof.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImmunityProof {
    threat_id: ThreatId,
    gene_hash: GeneHandle,
    commitment: Hash,
}

impl ImmunityProof {
    pub fn threat_id(&self) -> ThreatId {
        self.threat_id
    }

    pub fn gene_hash(&self) -> GeneHandle {
        self.gene_hash
    }
}

fn commitment_for(threat_id: &ThreatId, gene_hash: &GeneHandle) -> Hash {
    let mut hasher = Sha256::new();
    hasher.update(threat_id.0);
    hasher.update(gene_hash.0);
    hasher.update(b"neutralized+non-autoimmune");
    hasher.finalize().into()
}

/// Generates the proof — only possible once Stage 2 crashed the target in-sandbox
/// (`TrialOutcome::TargetCrashed`) and Stage 3's regression matrix passed
/// (`RegressionOutcome::Passed`). Anything short of that returns `None`, so a gene that
/// hasn't cleared both prior stages can't be attested to.
pub fn generate_proof(
    threat_id: ThreatId,
    gene: &GenePayload,
    sandbox_outcome: TrialOutcome,
    regression_outcome: RegressionOutcome,
) -> Option<ImmunityProof> {
    if sandbox_outcome != TrialOutcome::TargetCrashed || regression_outcome != RegressionOutcome::Passed {
        return None;
    }
    let gene_hash = gene.gene_hash();
    let commitment = commitment_for(&threat_id, &gene_hash);
    Some(ImmunityProof { threat_id, gene_hash, commitment })
}

/// Minimum validator signatures for Proof-of-Immunity consensus to commit a gene. The
/// Source of Truth names the mechanism but not a number (the same gap as
/// `registry::MOBILIZATION_THRESHOLD`) — 2-of-3 is the smallest majority that demonstrates
/// "network validators verify", tune when a real validator set size is known.
pub const POI_QUORUM: usize = 2;

/// One PoI consensus validator: recomputes the claimed commitment and signs only if it
/// matches — the mock stand-in for a real ZK verification-key check.
#[derive(Debug, Clone, Copy)]
pub struct Validator {
    pub id: &'static str,
}

impl Validator {
    fn verify_and_sign(&self, proof: &ImmunityProof) -> Option<String> {
        (commitment_for(&proof.threat_id, &proof.gene_hash) == proof.commitment)
            .then(|| format!("sig:{}", self.id))
    }
}

/// Mock network validator set standing in for the decentralized PoI consensus nodes.
pub const VALIDATORS: [Validator; 3] = [
    Validator { id: "validator-1" },
    Validator { id: "validator-2" },
    Validator { id: "validator-3" },
];

/// Result of submitting an [`ImmunityProof`] to the validator set.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConsensusOutcome {
    /// Fewer than [`POI_QUORUM`] validators signed — must not be committed.
    Rejected { signatures: Vec<String> },
    /// Quorum reached — ready for [`commit_gene`].
    Verified { signatures: Vec<String> },
}

/// Runs PoI consensus: every validator independently checks the proof and signs if it
/// verifies; quorum decides whether the network will commit it.
pub fn run_poi_consensus(proof: &ImmunityProof, validators: &[Validator]) -> ConsensusOutcome {
    let signatures: Vec<String> = validators
        .iter()
        .filter_map(|v| v.verify_and_sign(proof))
        .collect();
    if signatures.len() >= POI_QUORUM {
        ConsensusOutcome::Verified { signatures }
    } else {
        ConsensusOutcome::Rejected { signatures }
    }
}

/// Commits a gene that reached PoI consensus: publishes the `Threat_ID → Wasm_Gene_Hash` row
/// to the Genome Registry, then folds the updated registry into a new State Ledger block via
/// `conjugation`. Returns `None` for anything but `Verified` — a rejected proof has no path
/// to the ledger. Returns the new header for a light client to `adopt` (see ../state.rs).
pub fn commit_gene(
    consensus: &ConsensusOutcome,
    proof: &ImmunityProof,
    ipfs_uri: String,
    genome_registry: &mut GenomeRegistry,
    conjugation: &mut MockConjugationLink,
    threat_leaves: Vec<Hash>,
    timestamp: u64,
) -> Option<BlockHeader> {
    let ConsensusOutcome::Verified { signatures } = consensus else {
        return None;
    };
    genome_registry.publish(proof.threat_id, proof.gene_hash, ipfs_uri);
    let genome_leaves = genome_registry.gene_hashes();
    Some(conjugation.commit_block(threat_leaves, genome_leaves, signatures.clone(), timestamp))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::evolution::alleles::Allele;
    use crate::ledger::state::StateLedger;

    fn winning_gene() -> GenePayload {
        GenePayload { sequence: vec![Allele::Allele04, Allele::Allele12] }
    }

    #[test]
    fn proof_generation_refuses_a_gene_that_has_not_cleared_stages_2_and_3() {
        let threat_id = ThreatId([1u8; 32]);
        let gene = winning_gene();

        assert!(generate_proof(
            threat_id,
            &gene,
            TrialOutcome::TargetSurvived,
            RegressionOutcome::Passed
        )
        .is_none());

        assert!(generate_proof(
            threat_id,
            &gene,
            TrialOutcome::TargetCrashed,
            RegressionOutcome::AllergyFlagged("LegacyBackupAgent")
        )
        .is_none());
    }

    #[test]
    fn passing_gene_gets_a_verified_proof_that_commits_and_appears_on_the_ledger() {
        let threat_id = ThreatId([2u8; 32]);
        let gene = winning_gene();

        let proof = generate_proof(
            threat_id,
            &gene,
            TrialOutcome::TargetCrashed,
            RegressionOutcome::Passed,
        )
        .expect("gene cleared stage 2 and 3");

        let consensus = run_poi_consensus(&proof, &VALIDATORS);
        assert_eq!(
            consensus,
            ConsensusOutcome::Verified {
                signatures: vec![
                    "sig:validator-1".into(),
                    "sig:validator-2".into(),
                    "sig:validator-3".into(),
                ]
            }
        );

        let mut genome_registry = GenomeRegistry::new();
        let mut conjugation = MockConjugationLink::new();
        let header = commit_gene(
            &consensus,
            &proof,
            "ipfs://mock-gene".into(),
            &mut genome_registry,
            &mut conjugation,
            vec![],
            0,
        )
        .expect("verified consensus commits");

        let entry = genome_registry
            .get(&threat_id)
            .expect("committed record appears in the Genome Registry");
        assert_eq!(entry.gene_hash, proof.gene_hash());

        let mut state_ledger = StateLedger::new();
        state_ledger.adopt(header);
        let gene_proof = conjugation.genome_proof(0).unwrap();
        assert!(state_ledger.verify_gene(proof.gene_hash().0, &gene_proof));
    }

    #[test]
    fn tampered_proof_fails_poi_consensus_and_never_commits() {
        let threat_id = ThreatId([3u8; 32]);
        let gene = winning_gene();
        let mut proof = generate_proof(
            threat_id,
            &gene,
            TrialOutcome::TargetCrashed,
            RegressionOutcome::Passed,
        )
        .unwrap();
        proof.commitment = [0xFFu8; 32]; // claim doesn't match the (private) recomputation

        let consensus = run_poi_consensus(&proof, &VALIDATORS);
        assert_eq!(consensus, ConsensusOutcome::Rejected { signatures: vec![] });

        let mut genome_registry = GenomeRegistry::new();
        let mut conjugation = MockConjugationLink::new();
        let header = commit_gene(
            &consensus,
            &proof,
            "ipfs://mock-gene".into(),
            &mut genome_registry,
            &mut conjugation,
            vec![],
            0,
        );

        assert!(header.is_none(), "a rejected proof must never commit");
        assert!(genome_registry.get(&threat_id).is_none());
    }
}

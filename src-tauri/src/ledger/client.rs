//! P2P / WebSocket transport ("conjugation") between nodes.
//!
//! Real gossip/consensus networking is out of scope for the PoC (see
//! ../../.claude/docs/security.md); `MockConjugationLink` stands in for the wire so an
//! endpoint can exercise the exact commit / fetch-header / request-Merkle-path flow it will
//! use against a real peer. See ../../.claude/docs/plan.md, Phase 5.

use crate::ledger::registry::SuppressorToken;
use crate::ledger::state::{BlockHeader, Hash, MerkleProof, MerkleTree};

/// A network peer able to commit a block over the current registry contents and answer
/// Merkle-path requests for it. Stands in for the real conjugation transport.
#[derive(Debug)]
pub struct MockConjugationLink {
    headers: Vec<BlockHeader>,
    threat_tree: MerkleTree,
    genome_tree: MerkleTree,
    pending_suppressors: Vec<SuppressorToken>,
}

impl MockConjugationLink {
    pub fn new() -> Self {
        Self {
            headers: Vec::new(),
            threat_tree: MerkleTree::build(vec![]),
            genome_tree: MerkleTree::build(vec![]),
            pending_suppressors: Vec::new(),
        }
    }

    /// Commits a new block: rebuilds the Merkle trees over the given registry row hashes
    /// and appends a header. Returns the header for a light client to `adopt` (see
    /// ../state.rs).
    pub fn commit_block(
        &mut self,
        threat_leaves: Vec<Hash>,
        genome_leaves: Vec<Hash>,
        validator_signatures: Vec<String>,
        timestamp: u64,
    ) -> BlockHeader {
        self.threat_tree = MerkleTree::build(threat_leaves);
        self.genome_tree = MerkleTree::build(genome_leaves);
        let header = BlockHeader {
            height: self.headers.len() as u64,
            timestamp,
            threat_root: self.threat_tree.root(),
            genome_root: self.genome_tree.root(),
            validator_signatures,
        };
        self.headers.push(header.clone());
        header
    }

    pub fn latest_header(&self) -> Option<&BlockHeader> {
        self.headers.last()
    }

    /// Broadcasts an Epigenetic Suppressor Token onto the wire — the kill-switch, sent to
    /// every node. Peers pick it up with [`Self::drain_suppressors`].
    pub fn broadcast_suppressor(&mut self, token: SuppressorToken) {
        self.pending_suppressors.push(token);
    }

    /// Drains the suppressor tokens received since the last poll, so a node can apply them to
    /// its local Genome Registry (see `GenomeRegistry::apply_suppressor`).
    pub fn drain_suppressors(&mut self) -> Vec<SuppressorToken> {
        std::mem::take(&mut self.pending_suppressors)
    }

    /// Serves a Merkle path for the threat row at `index` in the most recently committed
    /// block — the "tiny cryptographic path" instead of the full Threat Registry.
    pub fn threat_proof(&self, index: usize) -> Option<MerkleProof> {
        self.threat_tree.proof(index)
    }

    /// Serves a Merkle path for the genome row at `index` in the most recently committed
    /// block.
    pub fn genome_proof(&self, index: usize) -> Option<MerkleProof> {
        self.genome_tree.proof(index)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ledger::state::StateLedger;

    #[test]
    fn endpoint_verifies_a_threat_via_a_fetched_path_only() {
        let mut link = MockConjugationLink::new();
        let leaves = vec![[1u8; 32], [2u8; 32], [3u8; 32]];
        let header = link.commit_block(leaves, vec![], vec!["validator-1".into()], 1);

        let mut endpoint = StateLedger::new();
        endpoint.adopt(header);

        let proof = link.threat_proof(1).unwrap();
        assert!(endpoint.verify_threat([2u8; 32], &proof));
    }

    #[test]
    fn broadcast_suppressor_is_drained_and_applied() {
        use crate::evolution::alleles::{Allele, GenePayload};
        use crate::ledger::registry::{
            BehavioralSchema, EpigeneticStatus, GenomeRegistry, MockIpfsStore, SuppressorToken,
        };

        let threat_id = BehavioralSchema(vec!["vssadmin".into()]).threat_id();
        let gene = GenePayload {
            sequence: vec![Allele::Allele04, Allele::Allele12],
        };
        let mut ipfs = MockIpfsStore::new();
        let uri = ipfs.store(gene.clone());
        let mut genome = GenomeRegistry::new();
        genome.publish(threat_id, gene.gene_hash(), uri);

        let mut link = MockConjugationLink::new();
        link.broadcast_suppressor(SuppressorToken { threat_id });

        // A node drains the wire and applies the received tokens to its local registry.
        for token in link.drain_suppressors() {
            genome.apply_suppressor(&token);
        }

        assert_eq!(
            genome.get(&threat_id).unwrap().epigenetic_status,
            EpigeneticStatus::Suppressed
        );
        assert!(
            link.drain_suppressors().is_empty(),
            "drained tokens are not redelivered"
        );
    }
}

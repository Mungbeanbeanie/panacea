//! Local Merkle-tree construction and proof verification against the State Ledger root.
//! Every threat or gene fetched from the network is untrusted until it verifies here.
//!
//! This is Ledger 1 (Phase 5 of ../../.claude/docs/plan.md): the only ledger an endpoint
//! downloads in full. Threat Registry and Genome Registry rows are verified via a
//! Merkle path (see ../client.rs), never a full-registry download.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// A tree node hash — the same 32-byte shape as `ThreatId`/`GeneHandle`, so registry rows
/// become leaves without a conversion step.
pub type Hash = [u8; 32];

fn hash_pair(left: &Hash, right: &Hash) -> Hash {
    let mut hasher = Sha256::new();
    hasher.update(left);
    hasher.update(right);
    hasher.finalize().into()
}

/// Which side a proof step's sibling sits on, so verification rebuilds each parent in the
/// right order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Side {
    Left,
    Right,
}

/// A Merkle authentication path: the sibling hash at each level from leaf to root. This is
/// the "tiny cryptographic path" a light client requests instead of the full registry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MerkleProof {
    steps: Vec<(Side, Hash)>,
}

impl MerkleProof {
    /// Recomputes the root from `leaf` plus this path and checks it against `root`.
    pub fn verify(&self, leaf: Hash, root: Hash) -> bool {
        let mut current = leaf;
        for (side, sibling) in &self.steps {
            current = match side {
                Side::Left => hash_pair(sibling, &current),
                Side::Right => hash_pair(&current, sibling),
            };
        }
        current == root
    }
}

/// A binary Merkle tree over registry row hashes (Threat Registry or Genome Registry). A
/// dangling last node at an odd level is paired with itself (standard Merkle padding).
#[derive(Debug, Clone)]
pub struct MerkleTree {
    levels: Vec<Vec<Hash>>,
}

impl MerkleTree {
    /// Builds a tree over `leaves`. An empty registry roots to an all-zero hash.
    pub fn build(leaves: Vec<Hash>) -> Self {
        if leaves.is_empty() {
            return Self {
                levels: vec![vec![[0u8; 32]]],
            };
        }
        let mut levels = vec![leaves];
        while levels.last().unwrap().len() > 1 {
            let prev = levels.last().unwrap();
            let mut next = Vec::with_capacity((prev.len() + 1) / 2);
            for pair in prev.chunks(2) {
                next.push(match pair {
                    [a, b] => hash_pair(a, b),
                    [a] => hash_pair(a, a),
                    _ => unreachable!(),
                });
            }
            levels.push(next);
        }
        Self { levels }
    }

    pub fn root(&self) -> Hash {
        self.levels.last().unwrap()[0]
    }

    /// Builds the authentication path for the leaf at `index`, or `None` if out of range.
    pub fn proof(&self, mut index: usize) -> Option<MerkleProof> {
        if index >= self.levels[0].len() {
            return None;
        }
        let mut steps = Vec::new();
        for level in &self.levels[..self.levels.len() - 1] {
            let sibling_index = index ^ 1;
            let sibling = level.get(sibling_index).copied().unwrap_or(level[index]);
            let side = if sibling_index < index { Side::Left } else { Side::Right };
            steps.push((side, sibling));
            index /= 2;
        }
        Some(MerkleProof { steps })
    }
}

/// One block header: chronological metadata plus the two registry roots it commits to.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockHeader {
    pub height: u64,
    pub timestamp: u64,
    pub threat_root: Hash,
    pub genome_root: Hash,
    /// Mock validator signatures — real signing/PoI consensus is Phase 8 (out of scope
    /// here); see ../../.claude/docs/security.md.
    pub validator_signatures: Vec<String>,
}

/// The State Ledger (Ledger 1) as an endpoint holds it: a chain of headers only, never the
/// full Threat/Genome registries. Verifying a threat or gene means fetching a Merkle path
/// from a peer (see ../client.rs) and checking it against a header trusted here.
#[derive(Debug, Default)]
pub struct StateLedger {
    headers: Vec<BlockHeader>,
}

impl StateLedger {
    pub fn new() -> Self {
        Self::default()
    }

    /// Adopts a header received from the network (e.g. via `MockConjugationLink`).
    pub fn adopt(&mut self, header: BlockHeader) {
        self.headers.push(header);
    }

    pub fn latest(&self) -> Option<&BlockHeader> {
        self.headers.last()
    }

    /// Verifies a Threat Registry row hash against the latest committed `threat_root`.
    pub fn verify_threat(&self, leaf: Hash, proof: &MerkleProof) -> bool {
        self.latest().is_some_and(|h| proof.verify(leaf, h.threat_root))
    }

    /// Verifies a Genome Registry row hash (a `Wasm_Gene_Hash`) against the latest
    /// committed `genome_root`.
    pub fn verify_gene(&self, leaf: Hash, proof: &MerkleProof) -> bool {
        self.latest().is_some_and(|h| proof.verify(leaf, h.genome_root))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn leaf(byte: u8) -> Hash {
        [byte; 32]
    }

    #[test]
    fn proof_verifies_every_leaf_in_an_odd_sized_tree() {
        let leaves = vec![leaf(1), leaf(2), leaf(3)];
        let tree = MerkleTree::build(leaves.clone());
        for (i, l) in leaves.into_iter().enumerate() {
            let proof = tree.proof(i).unwrap();
            assert!(proof.verify(l, tree.root()));
        }
    }

    #[test]
    fn tampered_leaf_fails_verification() {
        let tree = MerkleTree::build(vec![leaf(1), leaf(2), leaf(3), leaf(4)]);
        let proof = tree.proof(1).unwrap();
        assert!(!proof.verify(leaf(9), tree.root()));
    }

    #[test]
    fn light_client_verifies_without_the_full_registry() {
        let threat_tree = MerkleTree::build(vec![leaf(1), leaf(2)]);
        let header = BlockHeader {
            height: 0,
            timestamp: 0,
            threat_root: threat_tree.root(),
            genome_root: MerkleTree::build(vec![]).root(),
            validator_signatures: vec!["validator-1".into()],
        };

        let mut endpoint = StateLedger::new();
        endpoint.adopt(header);

        let proof = threat_tree.proof(1).unwrap();
        assert!(endpoint.verify_threat(leaf(2), &proof));
        assert!(!endpoint.verify_threat(leaf(9), &proof));
    }
}

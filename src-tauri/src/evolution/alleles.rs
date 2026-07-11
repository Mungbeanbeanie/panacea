//! Exploit-primitive (allele) matrix and the combinatorial fuzz driver that searches for a
//! combination which reliably aborts the target inside the sandbox, then compiles the
//! winning sequence to a Wasm gene payload.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::core::GeneHandle;
use crate::evolution::sandbox::{Sandbox, TrialOutcome};

/// A pre-compiled, safe exploit primitive the fuzz driver combines. Named after the
/// Source of Truth's worked example: `Allele04` (Thread-Context Exit Token Injection) +
/// `Allele12` (IPC Pipe Buffer Overflow) is the combo that reliably kills the target.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Allele {
    Allele04 = 0,
    Allele12 = 1,
    /// Deliberately too aggressive — destabilizes the mock host, so the fuzz driver must
    /// reject any combo containing it even when the target crashes.
    Allele09 = 2,
}

impl Allele {
    pub const CATALOG: [Allele; 3] = [Allele::Allele04, Allele::Allele12, Allele::Allele09];

    /// Decodes one byte of an on-chain `gene_seq` back into an allele, via the catalog
    /// rather than a separate match arm — the two can't drift out of sync.
    fn from_byte(byte: u8) -> Option<Self> {
        Self::CATALOG.iter().find(|allele| **allele as u8 == byte).copied()
    }
}

/// Compiled winning allele sequence: the Wasm Gene Payload a Soldier writes to the Genome
/// Registry (Ledger 3, Phase 6).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GenePayload {
    pub sequence: Vec<Allele>,
}

impl GenePayload {
    /// `Wasm_Gene_Hash`: cryptographic hash of the winning allele sequence.
    pub fn gene_hash(&self) -> GeneHandle {
        let mut hasher = Sha256::new();
        for allele in &self.sequence {
            hasher.update([*allele as u8]);
        }
        GeneHandle(hasher.finalize().into())
    }

    /// Encodes the sequence as raw bytes for the Genome Registry's `gene_seq` field — the
    /// gene's on-chain wire format now that it's stored directly in the account.
    pub fn to_bytes(&self) -> Vec<u8> {
        self.sequence.iter().map(|allele| *allele as u8).collect()
    }

    /// Decodes a `gene_seq` account field back into a runnable sequence. `None` if any byte
    /// isn't a known allele (corrupt/foreign data).
    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        let sequence = bytes
            .iter()
            .copied()
            .map(Allele::from_byte)
            .collect::<Option<Vec<_>>>()?;
        Some(Self { sequence })
    }
}

/// Every non-empty subset of the allele catalog, ascending by combination size, so the
/// fuzz driver tries the cheapest combos first.
fn candidate_combos() -> Vec<Vec<Allele>> {
    let catalog = Allele::CATALOG;
    let mut combos: Vec<Vec<Allele>> = (1u32..(1 << catalog.len()))
        .map(|mask| {
            catalog
                .iter()
                .enumerate()
                .filter(|(i, _)| mask & (1 << i) != 0)
                .map(|(_, allele)| *allele)
                .collect()
        })
        .collect();
    combos.sort_by_key(|combo| combo.len());
    combos
}

/// Combinatorial fuzz driver: tries allele combinations against the sandboxed target,
/// smallest first, until one crashes the target without destabilizing the mock host, then
/// compiles that sequence into a [`GenePayload`]. Returns `None` if the whole catalog is
/// exhausted without a safe kill.
pub fn fuzz(sandbox: &mut Sandbox) -> Option<GenePayload> {
    for combo in candidate_combos() {
        if sandbox.run(&combo) == TrialOutcome::TargetCrashed {
            return Some(GenePayload { sequence: combo });
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::evolution::sandbox::FrozenProcess;

    #[test]
    fn fuzz_finds_the_safe_kill_combo() {
        let mut sandbox = Sandbox::spawn(FrozenProcess {
            pid: 1,
            memory: vec![],
        });
        let gene = fuzz(&mut sandbox).expect("fuzz driver should find a winning combo");
        assert_eq!(gene.sequence, vec![Allele::Allele04, Allele::Allele12]);
    }

    #[test]
    fn gene_bytes_round_trip() {
        let gene = GenePayload {
            sequence: vec![Allele::Allele04, Allele::Allele12],
        };
        let decoded = GenePayload::from_bytes(&gene.to_bytes()).expect("valid bytes decode");
        assert_eq!(decoded, gene);
    }

    #[test]
    fn corrupt_gene_bytes_fail_to_decode() {
        assert!(GenePayload::from_bytes(&[0xFF]).is_none());
    }

    #[test]
    fn same_sequence_hashes_identically() {
        let a = GenePayload {
            sequence: vec![Allele::Allele04, Allele::Allele12],
        };
        let b = GenePayload {
            sequence: vec![Allele::Allele04, Allele::Allele12],
        };
        assert_eq!(a.gene_hash(), b.gene_hash());
    }
}

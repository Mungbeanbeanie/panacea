//! Exploit-primitive (allele) matrix and the combinatorial fuzz driver that searches for a
//! combination which reliably aborts the target inside the sandbox, then compiles the
//! winning sequence into a real, portable Wasm binary ([`CompiledGene`]).

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
}

/// A fuzz-found candidate: the allele sequence the Lymph Node's allergy check (Stage 3)
/// inspects before anything is published. In-memory only — never itself round-tripped
/// through the ledger. [`GenePayload::compile`] derives the real executable artifact
/// ([`CompiledGene`]) that actually travels through the Genome Registry.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GenePayload {
    pub sequence: Vec<Allele>,
}

/// Fixed WAT skeleton for one compiled gene: a zero-argument `run` export whose body is
/// this specific gene's already-decided bitmask, baked in as a constant (`{bitmask}`,
/// templated via `str::replace` — WAT syntax uses no literal braces, so this is safe).
/// Same synthetic physics as `evolution::sandbox::EVALUATE_WAT` (bit 2 set →
/// `HostDestabilized`; bits 0+1 both set → `TargetCrashed`; otherwise `TargetSurvived`),
/// just expressed as a per-gene constant instead of a runtime parameter, since a committed
/// gene *is* one specific, already-decided combo rather than a general evaluator.
const GENE_WAT_TEMPLATE: &str = r#"
(module
  (func $run (export "run") (result i32)
    (local $bitmask i32)
    (local.set $bitmask (i32.const {bitmask}))
    (if (i32.ne (i32.and (local.get $bitmask) (i32.const 4)) (i32.const 0))
      (then (return (i32.const 2))))
    (if (i32.eq (i32.and (local.get $bitmask) (i32.const 3)) (i32.const 3))
      (then (return (i32.const 1))))
    (i32.const 0)
  )
)
"#;

impl GenePayload {
    /// Compiles this candidate into the real, portable `.wasm` binary that becomes the
    /// on-chain Wasm Gene Payload — the executable artifact `Sandbox::run_gene` runs and the
    /// Genome Registry stores, as opposed to this in-memory allele list.
    pub fn compile(&self) -> CompiledGene {
        let bitmask: u32 = self
            .sequence
            .iter()
            .fold(0u32, |mask, allele| mask | (1 << *allele as u8));
        let wat = GENE_WAT_TEMPLATE.replace("{bitmask}", &bitmask.to_string());
        let wasm_bytes = wat::parse_str(&wat)
            .expect("GENE_WAT_TEMPLATE is a fixed, hand-written constant validated at compile time");
        CompiledGene { wasm_bytes }
    }
}

/// The real executable artifact: a compiled `.wasm` binary, hashed as the `Wasm_Gene_Hash`
/// and stored directly in the Genome Registry's `gene_seq` field. Unlike [`GenePayload`],
/// this can't be decompiled back into an allele list — nothing downstream needs to: once a
/// gene is committed, every later consumer (hash verification, sandbox execution) only ever
/// needs the bytes, never which alleles produced them.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompiledGene {
    pub wasm_bytes: Vec<u8>,
}

impl CompiledGene {
    /// `Wasm_Gene_Hash`: cryptographic hash of the compiled Wasm binary.
    pub fn gene_hash(&self) -> GeneHandle {
        let mut hasher = Sha256::new();
        hasher.update(&self.wasm_bytes);
        GeneHandle(hasher.finalize().into())
    }

    /// The Genome Registry's `gene_seq` wire format — the compiled binary itself.
    pub fn to_bytes(&self) -> Vec<u8> {
        self.wasm_bytes.clone()
    }

    /// Decodes a `gene_seq` account field, validating it actually parses as a real Wasm
    /// module (the same validation `wasmi::Module::new` performs). `None` if the bytes are
    /// corrupt or foreign data.
    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        let engine = wasmi::Engine::default();
        wasmi::Module::new(&engine, bytes).ok()?;
        Some(Self {
            wasm_bytes: bytes.to_vec(),
        })
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
    fn fuzz_on_hardened_target_finds_the_allergic_allele() {
        // The hardened strain only dies to Allele09 — the candidate Stage 3 must then
        // allergy-flag (LegacyBackupAgent), so this is the allergy demo's Stage-2 half.
        let mut sandbox = Sandbox::spawn_hardened(FrozenProcess {
            pid: 1,
            memory: vec![],
        });
        let gene = fuzz(&mut sandbox).expect("fuzz driver should find the aggressive combo");
        assert_eq!(gene.sequence, vec![Allele::Allele09]);
    }

    #[test]
    fn compiled_gene_bytes_round_trip() {
        let gene = GenePayload {
            sequence: vec![Allele::Allele04, Allele::Allele12],
        };
        let compiled = gene.compile();
        let decoded = CompiledGene::from_bytes(&compiled.to_bytes()).expect("valid bytes decode");
        assert_eq!(decoded, compiled);
    }

    #[test]
    fn corrupt_compiled_gene_bytes_fail_to_decode() {
        assert!(CompiledGene::from_bytes(&[0xFF]).is_none());
    }

    #[test]
    fn same_sequence_compiles_and_hashes_identically() {
        let a = GenePayload {
            sequence: vec![Allele::Allele04, Allele::Allele12],
        };
        let b = GenePayload {
            sequence: vec![Allele::Allele04, Allele::Allele12],
        };
        assert_eq!(a.compile().gene_hash(), b.compile().gene_hash());
    }

    #[test]
    fn compiled_gene_is_valid_wasm_with_one_zero_arg_export() {
        let gene = GenePayload {
            sequence: vec![Allele::Allele04, Allele::Allele12],
        };
        let compiled = gene.compile();

        let engine = wasmi::Engine::default();
        let module =
            wasmi::Module::new(&engine, &compiled.wasm_bytes[..]).expect("compiled gene is valid Wasm");
        let exports: Vec<_> = module.exports().collect();
        assert_eq!(exports.len(), 1, "exactly one export");
        assert_eq!(exports[0].name(), "run");
    }

    #[test]
    fn compiled_gene_has_zero_imports() {
        // Same isolation proof as sandbox::tests::compiled_gene_module_has_zero_imports, but
        // for the per-gene templated module this type actually produces (that test only
        // covers the shared EVALUATE_WAT module) — a compiled gene must be just as
        // incapable of touching the real host, regardless of which specific combo it bakes in.
        let gene = GenePayload {
            sequence: vec![Allele::Allele09],
        };
        let compiled = gene.compile();

        let engine = wasmi::Engine::default();
        let module =
            wasmi::Module::new(&engine, &compiled.wasm_bytes[..]).expect("compiled gene is valid Wasm");
        assert_eq!(
            module.imports().count(),
            0,
            "a compiled gene must not import any host function"
        );
    }
}

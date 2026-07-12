use anchor_lang::prelude::*;

/// PDA seed prefix for a Threat Registry row (Ledger 2). Combined with `threat_id`.
#[constant]
pub const THREAT_SEED: &[u8] = b"threat";

/// PDA seed prefix for a Genome Registry row (Ledger 3). Combined with `threat_id`.
#[constant]
pub const GENOME_SEED: &[u8] = b"genome";

/// Confidence score that triggers a network-wide mobilization command. Mirrors
/// `ledger::registry::MOBILIZATION_THRESHOLD` in the Rust core mock — the Source of Truth
/// names the mechanism but not a number; kept in sync with the existing mock's value.
pub const MOBILIZATION_THRESHOLD: u32 = 2;

/// Minimum Lymph Node signatures for Proof of Immunity (source-of-truth.md: 3-of-5).
pub const POI_QUORUM: u8 = 3;

/// Longest compiled Wasm gene binary a `GenomeEntry.gene_seq` can hold. The Rust core
/// compiles each winning allele combo into a real `.wasm` binary (`evolution/alleles.rs`'s
/// `GenePayload::compile`/`CompiledGene`) rather than storing a raw allele-index array;
/// measured via `wat::parse_str` on the fixed WAT skeleton this codebase generates: ~104
/// bytes. 256 gives comfortable headroom over that measurement without being an arbitrary
/// round number picked with no basis.
pub const GENE_SEQ_MAX_LEN: usize = 256;

/// The 5 Lymph Node validators' persistent Solana pubkeys gating `commit_gene` — hardcoded
/// rather than stored in a mutable account, matching how every other threshold in this
/// codebase (`ANOMALY_THRESHOLD`, `MOBILIZATION_THRESHOLD`, `POI_QUORUM` in the Rust core
/// mock) is a compile-time constant next to the logic that checks it. Must match the
/// `../../../keys/lymph-nodes/validator-{1..5}.json` keypairs the Rust core signs with
/// (`agents::scout::spawn_demo`) — these are those 5 files' actual pubkeys (a Phase-11 bug:
/// the two had drifted apart, so no real signature set could ever clear `POI_QUORUM`; caught
/// running the live devnet test this Phase-12 pass finally exercises end to end).
pub const LYMPH_NODE_VALIDATORS: [Pubkey; 5] = [
    Pubkey::new_from_array([
        236, 36, 117, 118, 232, 139, 161, 20, 144, 34, 32, 77, 96, 156, 62, 210, 107, 162, 92,
        102, 91, 139, 36, 189, 158, 95, 143, 26, 37, 79, 73, 130,
    ]),
    Pubkey::new_from_array([
        151, 105, 176, 159, 98, 151, 116, 211, 241, 42, 125, 59, 154, 37, 128, 190, 71, 129, 65,
        145, 200, 162, 43, 144, 126, 88, 5, 44, 254, 114, 236, 64,
    ]),
    Pubkey::new_from_array([
        37, 79, 194, 64, 145, 190, 228, 148, 0, 189, 130, 246, 210, 50, 68, 213, 114, 80, 195, 72,
        152, 47, 56, 21, 90, 99, 253, 45, 98, 175, 250, 77,
    ]),
    Pubkey::new_from_array([
        225, 151, 229, 188, 196, 102, 157, 198, 248, 64, 224, 196, 142, 216, 68, 11, 222, 193, 80,
        77, 21, 154, 99, 161, 49, 250, 175, 2, 108, 189, 139, 48,
    ]),
    Pubkey::new_from_array([
        153, 83, 179, 119, 129, 117, 124, 118, 11, 51, 129, 181, 45, 191, 174, 214, 154, 59, 126,
        12, 151, 25, 233, 220, 134, 73, 115, 140, 64, 3, 150, 192,
    ]),
];

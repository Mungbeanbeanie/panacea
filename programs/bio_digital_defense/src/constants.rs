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

/// The 5 Lymph Node validators' persistent Solana pubkeys gating `commit_gene` — hardcoded
/// rather than stored in a mutable account, matching how every other threshold in this
/// codebase (`ANOMALY_THRESHOLD`, `MOBILIZATION_THRESHOLD`, `POI_QUORUM` in the Rust core
/// mock) is a compile-time constant next to the logic that checks it.
pub const LYMPH_NODE_VALIDATORS: [Pubkey; 5] = [
    Pubkey::new_from_array([
        142, 82, 206, 242, 93, 194, 34, 43, 59, 80, 27, 93, 244, 85, 198, 52, 231, 35, 156, 51,
        35, 78, 243, 0, 236, 173, 118, 223, 15, 135, 17, 142,
    ]),
    Pubkey::new_from_array([
        219, 89, 96, 61, 106, 39, 67, 47, 122, 192, 94, 179, 26, 82, 186, 215, 168, 204, 41, 194,
        134, 78, 34, 95, 31, 51, 112, 43, 244, 238, 173, 99,
    ]),
    Pubkey::new_from_array([
        123, 96, 229, 72, 228, 82, 72, 254, 169, 203, 144, 51, 233, 230, 83, 225, 17, 34, 233, 36,
        226, 191, 224, 8, 98, 214, 135, 253, 17, 223, 220, 174,
    ]),
    Pubkey::new_from_array([
        224, 46, 94, 20, 218, 33, 253, 55, 227, 107, 232, 245, 235, 20, 253, 8, 198, 29, 90, 232,
        253, 178, 146, 178, 74, 220, 73, 49, 255, 109, 249, 114,
    ]),
    Pubkey::new_from_array([
        96, 255, 220, 23, 13, 174, 27, 138, 26, 181, 208, 106, 219, 132, 178, 123, 216, 6, 201,
        60, 15, 246, 89, 213, 237, 125, 22, 141, 249, 52, 187, 184,
    ]),
];

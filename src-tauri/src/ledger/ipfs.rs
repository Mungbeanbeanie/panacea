//! Real IPFS gene storage via Pinata's pinning service (Solana migration Phase 11, box 10).
//! Uploads a compiled Wasm Gene Payload when a cure is committed and fetches it by CID
//! before a Soldier runs it in-sandbox — replacing the in-memory `MockIpfsStore`.

use serde::Deserialize;

use crate::core::GeneHandle;
use crate::evolution::alleles::GenePayload;
use crate::ledger::LedgerError;

const PINATA_PIN_JSON_URL: &str = "https://api.pinata.cloud/pinning/pinJSONToIPFS";
const PINATA_GATEWAY: &str = "https://gateway.pinata.cloud/ipfs";
const PINATA_JWT_ENV: &str = "PINATA_JWT";

#[derive(Debug, Deserialize)]
struct PinResponse {
    #[serde(rename = "IpfsHash")]
    ipfs_hash: String,
}

/// Content-addressed identifier for a gene binary pinned on IPFS — the Genome Registry's
/// `IPFS_CID`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Cid(pub String);

/// Real IPFS pinning-service client (Pinata). Reads its API key from `PINATA_JWT` at call
/// time rather than caching it, so a missing key fails loudly per call instead of silently
/// at startup.
pub struct IpfsClient {
    http: reqwest::blocking::Client,
}

impl Default for IpfsClient {
    fn default() -> Self {
        Self::new()
    }
}

impl IpfsClient {
    pub fn new() -> Self {
        Self {
            http: reqwest::blocking::Client::new(),
        }
    }

    fn jwt() -> Result<String, LedgerError> {
        std::env::var(PINATA_JWT_ENV).map_err(|_| LedgerError::MissingPinataJwt)
    }

    /// Uploads a compiled gene payload, returning the CID to record as the Genome
    /// Registry's `IPFS_CID`. Called from the `commit_gene` client flow, before the
    /// on-chain instruction is submitted.
    pub fn upload(&self, gene: &GenePayload) -> Result<Cid, LedgerError> {
        let jwt = Self::jwt()?;
        let body = serde_json::json!({ "pinataContent": gene });

        let response = self
            .http
            .post(PINATA_PIN_JSON_URL)
            .bearer_auth(jwt)
            .json(&body)
            .send()?
            .error_for_status()?;

        let parsed: PinResponse = response.json()?;
        Ok(Cid(parsed.ipfs_hash))
    }

}

/// The gene-fetch boundary `resolve_and_run` depends on (Phase 11 box 11, Q8) — the same
/// reasoning as `GenomeSource` in `../registry.rs`: a real `IpfsClient` backs the live path,
/// `FakeGeneStore` backs the existing offline tests without hitting the network.
pub trait GeneStore {
    /// Fetches gene bytecode by CID, verifying it against `expected_hash` before handing it
    /// back — this comparison *is* the verification step now (see `../state.rs`'s
    /// commitment-level reads), replacing the old Merkle-proof check.
    fn fetch(&self, cid: &Cid, expected_hash: GeneHandle) -> Result<GenePayload, LedgerError>;
}

impl GeneStore for IpfsClient {
    fn fetch(&self, cid: &Cid, expected_hash: GeneHandle) -> Result<GenePayload, LedgerError> {
        let url = format!("{PINATA_GATEWAY}/{}", cid.0);
        let gene: GenePayload = self.http.get(&url).send()?.error_for_status()?.json()?;

        if gene.gene_hash() != expected_hash {
            return Err(LedgerError::GeneHashMismatch);
        }
        Ok(gene)
    }
}

/// Offline test double for `GeneStore` — an in-memory map, so `pharmacy_flow_tests`/
/// `live_dispense_tests` keep asserting fetch counts without a real Pinata account (Q8).
#[derive(Debug, Default)]
pub struct FakeGeneStore {
    blobs: std::collections::HashMap<String, GenePayload>,
    fetch_calls: std::cell::Cell<u32>,
}

impl FakeGeneStore {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn store(&mut self, cid: &str, gene: GenePayload) {
        self.blobs.insert(cid.to_string(), gene);
    }

    pub fn fetch_calls(&self) -> u32 {
        self.fetch_calls.get()
    }
}

impl GeneStore for FakeGeneStore {
    fn fetch(&self, cid: &Cid, expected_hash: GeneHandle) -> Result<GenePayload, LedgerError> {
        self.fetch_calls.set(self.fetch_calls.get() + 1);
        let gene = self
            .blobs
            .get(&cid.0)
            .cloned()
            .ok_or(LedgerError::GeneNotFound)?;
        if gene.gene_hash() != expected_hash {
            return Err(LedgerError::GeneHashMismatch);
        }
        Ok(gene)
    }
}

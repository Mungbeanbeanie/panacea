# Tech Stack

If you need a stack detail not stated here, read the manifest or config file rather than
inferring it (Working Agreement, Rule 4).

- **Desktop shell:** Tauri 2.x (Rust host + webview UI). Config lives in
  `src-tauri/tauri.conf.json`; permissions use Tauri 2's capability model.
- **Core language:** Rust (edition and toolchain pinned in `Cargo.toml` / `rust-toolchain`).
- **Frontend:** React with Vite, JSX (no TypeScript unless `package.json` says otherwise —
  check before assuming).
- **Sandboxing:** MicroVM / lightweight Wasm isolation (exact runtime defined in
  `evolution/sandbox.rs`).
- **Ledger / on-chain program:** Solana (devnet). The Threat and Genome Registries are an
  Anchor program in its own workspace (`programs/`, separate from `src-tauri/` — see
  [File Structure](file-structure.md)). Local nodes are RPC light clients via the
  `anchor-client` crate (blocking API — no `tokio` ripples through the agent core; reads
  at `confirmed` commitment) — no local validator. Each endpoint holds a devnet keypair to
  sign its own transactions (funded via faucet; never real funds). Lymph Node validators
  hold separate persistent keypairs used as the 3-of-5 multisig co-signers gating
  `commit_gene`/`suppress_gene` ("Proof of Immunity" — see
  [Source of Truth](source-of-truth.md)); they sign only, so they need no faucet funding of
  their own.
- **Gene storage:** the compiled allele sequence lives directly in the Genome Registry
  account (`gene_seq: Vec<u8>`, `programs/bio_digital_defense/src/state.rs`) — no off-chain
  blob store; a gene is a few bytes, smaller than a content address would be. Hash-verified
  against the account's own `Wasm_Gene_Hash` before a Soldier runs it.
- **What's still simulated:** the malware/virus itself (synthetic fixtures only — see
  [Security & Safety](security.md)) and the MicroVM/Wasm sandbox host. The chain,
  consensus finality, and on-chain program are real.

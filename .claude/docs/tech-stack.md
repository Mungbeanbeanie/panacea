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
  [File Structure](file-structure.md)). Local nodes are RPC light clients (Rust
  `solana-client`/`solana-sdk` crates) — no local validator. Each endpoint holds a devnet
  keypair to sign its own transactions (funded via faucet; never real funds). Lymph Node
  validators hold separate persistent keypairs used as the 3-of-5 multisig signers gating
  `commit_gene`/`suppress_gene` ("Proof of Immunity" — see
  [Source of Truth](source-of-truth.md)).
- **Gene storage:** real IPFS via a pinning service (e.g. web3.storage/Pinata HTTP API) —
  content-addressed, hash-verified against the on-chain `Wasm_Gene_Hash` before a Soldier
  runs it.
- **What's still simulated:** the malware/virus itself (synthetic fixtures only — see
  [Security & Safety](security.md)) and the MicroVM/Wasm sandbox host. The chain,
  consensus finality, on-chain program, and gene storage are real.

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
- **Ledger / P2P:** decentralized light-client model over WebSocket/P2P; IPFS for gene
  binary storage; ZK-proofs for cure verification. *(PoC: these may be mocked/stubbed —
  see [Architecture](architecture.md) Stage 4.)*

# Domain Glossary

Map every metaphor to its engineering meaning. Reason with the right column.

| Term | Engineering meaning |
|---|---|
| **Scout** | Lightweight background daemon doing behavioral process tracing |
| **Soldier** | Dormant on-disk remediation payload; wakes only on a high-confidence signal |
| **Spore** | A Soldier at rest — serialized, un-executed, resource-free |
| **Apoptosis** | Programmed self-cleanup: teardown / re-serialize to spore |
| **Trajectory / Trajectory score** | Weighted sum over an observed sequence of syscalls/actions |
| **Allele** | A safe, pre-compiled exploit primitive used by the fuzzer |
| **Wasm Gene / Gene payload** | Compiled, sandboxed remediation routine (WebAssembly bytecode) |
| **Lymph Node** | Privileged network validator running the regression matrix |
| **Allergy / Allergy flag** | A cure that harms a legitimate whitelisted application |
| **Proof of Immunity (PoI)** | The 3-of-5 Lymph Node multisig threshold, enforced on-chain by the Anchor program, gating a `commit_gene`/`suppress_gene` write — an app-level gate layered on top of Solana's own consensus, not a replacement for it |
| **Conjugation** | Solana RPC calls / signed transaction submission between a local node and the network |
| **Devnet** | Solana's public test network — real consensus and finality, no real funds |
| **Lymph Node keypair** | A Lymph Node validator's persistent Solana signing key; one of the 3-of-5 PoI multisig signers |
| **Anchor program** | The on-chain smart contract implementing the Threat and Genome Registries (`programs/`) |
| **PDA (Program Derived Address)** | The deterministic on-chain account address holding one Threat Registry or Genome Registry row |
| **Epigenetic status** | Active/suppressed flag on a gene (0 active, 1 suppressed) |
| **Epigenetic Suppressor Token** | Network kill-switch that disables a harmful cure instantly |

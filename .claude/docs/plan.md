# Build Roadmap

A discrete, phased checklist for the hackathon build. It conforms to
[source-of-truth.md](source-of-truth.md) (canonical) — thresholds, stage boundaries, and
ledger fields come from there. PoC bar throughout: **compiles + demo works** (see
[Build, Run, Test](build-run-test.md)). The chain (Solana devnet), consensus finality, the
on-chain Anchor program, PoI's multisig gate, and IPFS gene storage are all real — only the
malware/virus itself and the MicroVM/Wasm sandbox host stay simulated.

**How to use this file:** check off `- [ ]` boxes as build tasks land. Owners (A/B/C, see
[Collaboration](collaboration.md)) mostly edit their own phase's boxes, so completion
tracking stays low-collision. This tracks coarse build-task completion; fine-grained live
assignment lives in GitHub issues.

## Concurrency & dependencies (branch strategy)

Most phases live in disjoint directories (`agents/` vs `evolution/` vs `ledger/` vs
`landing/`), so they're safe on separate branches at the same time. True dependencies exist
only at integration points — and each is **decoupled with a mock interface** (the PoC
approach anyway), so a downstream teammate is never hard-blocked: build against the mock,
then swap to the real module when it merges.

```
Wave 0 (blocking, land together):   [0]
Wave 1 (fully parallel branches):   [1] [2] [3] [4] [5] [10]
Wave 2 (unlock as deps merge):      [6]←5      [7]←4
Wave 3 (integration):               [9]←6,3    [8]←6,7
Wave 4 (real backbone):             [11]←5,6,8,9
```

| Phase | Owner | Depends on (hard) | Parallel-safe after P0? | Decouple via |
|---|---|---|---|---|
| 0 Scaffolding | all | — | must be first | — |
| 1 Scouts / Stage 1 | A | 0 | ✅ | shared `core` types |
| 2 Threat Ledger | B | 0 | ✅ | mock `Threat_ID`/schema |
| 3 Soldier lifecycle | A | 0 | ✅ | mock wake signal |
| 4 Evolution / Stage 2 | B | 0 | ✅ | mock frozen process |
| 5 State Ledger | B | 0 | ✅ | self-contained |
| 6 Genome Ledger | B | 0, **5** | after 5 | mock gene payloads |
| 7 Stage 3 Lymph Node | A+B | 0, **4** | after 4 (or mock gene) | mock candidate gene |
| 8 Stage 4 Consensus | A+B | 0, **6**, **7** | after 6+7 | mock proof |
| 9 Kill-switch | B | 0, **6**, **3** | after 6 | — |
| 10 Dashboard | C | 0 | ✅ (parallel throughout) | mock event streams |
| 11 Solana migration | B | 0, **5**, **6**, **8**, **9** | after 5+6+8+9 | — (replaces the mocks those phases left in place) |

**Merge-risk caveats (beyond Phase 0's shared hotspots):**
- `ledger/registry.rs` holds **both** the Threat (Phase 2) and Genome (Phase 6) tables —
  same file, two phases. Sequence them under owner B, or split `registry.rs` into
  threat/genome sections early so the two branches touch different regions.
- Phases 1 and 3 are both owner A in `agents/`, but different files (`scout.rs` vs
  `soldier.rs`) — separate branches are fine.
- Phase 0's shared hotspots (`main.rs` registration, `Cargo.toml`, `core/mod.rs` types) are
  why P0 lands together, first, before any branch diverges.

---

## Phase 0 — Scaffolding & Foundations
**Owner:** all (together, first) · **Depends on:** — · **Parallel-safe:** must be first

- [x] `cargo tauri init` in `src-tauri/` with pinned Tauri 2 + crate versions
- [x] Populate `landing/` deps (React, Vite, `@tauri-apps/api`) + lockfile
- [x] Validate `tauri.conf.json` for the sibling `landing/` + `src-tauri/` layout
- [x] `.github/` CI: `cargo check` + `cargo build` + frontend build per platform
- [x] Whole module tree compiles — `cargo check` green
- [x] Shared cross-cutting types in `core/mod.rs` (`Threat_ID`, anomaly score, PID, gene handle)

**Done when:** `npm run tauri dev` opens the window with an empty dashboard.

## Phase 1 — Scout Agents & Stage 1 (Trajectory Scoring)
**Owner:** A · **Depends on:** 0 · **Parallel-safe:** ✅ · **Decouple via:** shared `core` types

- [x] Ultra-light background daemon loop (`agents/scout.rs`)
- [x] Behavioral tracing: syscall anomalies, memory-space boundary violations, I/O bursts
- [x] State-transition matrix: Action A +20, Action B +50, Action C +40
- [x] Per-PID cumulative trajectory score
- [x] 100-pt threshold → hard interrupt: suspend all threads of the target PID
- [x] Signal the local Soldier spore on threshold cross
- [x] Emit `Behavioral_Schema` (syscall/port sequence) for the Threat Registry

**Done when:** a scripted "bad" process crosses 100 pts, is suspended, and the spore is signaled.

## Phase 2 — Threat Registry (Ledger 2)
**Owner:** B · **Depends on:** 0 · **Parallel-safe:** ✅ · **Decouple via:** mock `Threat_ID`/schema

- [x] `Threat_ID` = cryptographic hash of the behavioral vector
- [x] Store `Behavioral_Schema`
- [x] `Confidence_Score` integer, incremented on matching trajectory
- [x] Correlate an incoming vector to an existing `Threat_ID`
- [x] Trigger network-wide mobilization once the confidence threshold is crossed

**Done when:** two matching trajectories increment the score and fire a (mocked) mobilization.

## Phase 3 — Soldier Spore Lifecycle
**Owner:** A · **Depends on:** 0 · **Parallel-safe:** ✅ · **Decouple via:** mock wake signal

- [x] Soldier at rest = dormant, serialized, un-executed spore on disk
- [x] Wake on a high-confidence threat notification (`Threat_ID`)
- [x] Clone the frozen process's memory space
- [x] Apoptosis: programmed deletion / re-serialize back to a passive spore after acting

**Done when:** a wake signal spins up a Soldier that acts, then re-serializes to a spore.

## Phase 4 — Evolution & Stage 2 (Local Isolation & Fuzzing)
**Owner:** B · **Depends on:** 0 · **Parallel-safe:** ✅ · **Decouple via:** mock frozen process

- [x] MicroVM/Wasm sandbox setup + teardown (`evolution/sandbox.rs`)
- [x] Clone the target into the sandbox against a mock host OS
- [x] Allele matrix — pre-compiled structural primitives (e.g. `Allele_04`, `Allele_12`)
- [x] Combinatorial fuzz driver trying allele combinations
- [x] Success criterion: target aborts/crashes without destabilizing the mock host
- [x] Compile the winning sequence → Wasm Gene Payload
- [x] Isolation guarantee: alleles/genes execute **only** in-sandbox (keep `sandbox-isolation-check` passing)

**Done when:** the fuzz finds a combo that kills the target in-sandbox and emits a Wasm gene.

## Phase 5 — State Ledger (Ledger 1) & Light-Client Verification
**Owner:** B · **Depends on:** 0 · **Parallel-safe:** ✅ · **Decouple via:** self-contained

- [x] Block headers, timestamps, validator signatures (`ledger/state.rs`)
- [x] Merkle roots of the Threat + Genome registries
- [x] Endpoint downloads only the State Ledger
- [x] Merkle-path request + verify against the local root
- [x] Conjugation transport (P2P/WebSocket), mockable (`ledger/client.rs`)

**Done when:** a gene/threat hash verifies against a Merkle root via a fetched path — no full-chain download.

## Phase 6 — Genome Registry (Ledger 3) & the Pharmacy Flow
**Owner:** B · **Depends on:** 0, **5** · **Parallel-safe:** after 5 · **Decouple via:** mock gene payloads

- [x] Mapping `Threat_ID → Wasm_Gene_Hash`
- [x] `IPFS_URI` for the compiled gene binary (mock IPFS store OK)
- [x] `Epigenetic_Status` flag (0 active / 1 suppressed)
- [x] Soldier queries Ledger 3 **on demand** by `Threat_ID` (never a passive scan)
- [x] Verify `Wasm_Gene_Hash` against the State Ledger Merkle root before use
- [x] Fetch bytecode from IPFS → run in sandbox → apoptosis

**Done when:** a Soldier resolves `Threat_ID` → verified gene → runs in-sandbox → apoptosis.

## Phase 7 — Stage 3 (Lymph Node Regression / Allergy Check)
**Owner:** A+B · **Depends on:** 0, **4** · **Parallel-safe:** after 4 (or mock gene) · **Decouple via:** mock candidate gene

- [x] Lymph Node validator environment: standard OS base + Top-5,000 apps (mock subset)
- [x] Execute the proposed Wasm mutation in the crowded environment
- [x] Detect a whitelisted app crash / memory leak → raise `Allergy Flag` → drop the gene

**Done when:** a gene that breaks a whitelisted app is flagged allergic and dropped.

## Phase 8 — Stage 4 (Consensus & Ledger Commitment)
**Owner:** A+B · **Depends on:** 0, **6**, **7** · **Parallel-safe:** after 6+7 · **Decouple via:** mock proof

- [x] Generate a ZK-Proof: gene neutralizes the threat + passed the allergy check, without exposing host/malware (mock)
- [x] Validators verify via Proof of Immunity consensus (mock)
- [x] Commit the gene to the blockchain (Genome Registry entry + State Ledger root update)

**Done when:** a passing gene produces a (mock) proof, is verified, and a committed record appears.

## Phase 9 — Epigenetic Suppression Kill-Switch
**Owner:** B · **Depends on:** 0, **6**, **3** · **Parallel-safe:** after 6 · first-class safety

- [x] Epigenetic Suppressor Token broadcast → set `Epigenetic_Status = 1` on a Gene ID
- [x] Soldiers reading Ledger 3 immediately stop executing that cure
- [x] Status check happens **before** any gene fetch/exec
- [x] Keep [`suppression-path-test`](../skills/suppression-path-test/SKILL.md) passing

**Done when:** flipping `Epigenetic_Status` to 1 halts the cure in seconds, before any fetch/exec.

## Phase 10 — Observability Dashboard
**Owner:** C · **Depends on:** 0 · **Parallel-safe:** ✅ (parallel throughout) · **Decouple via:** mock event streams

- [x] Wire `useTauriEvents` to real Rust event streams (mock streams until each phase lands)
- [x] `EcosystemGraph` — live process/node map
- [x] `LedgerTerminal` — rolling Proof-of-Immunity event log
- [x] `StrainTree` — evolutionary phylogeny of strains
- [x] Every live view handles the no-data / stream-dropped state

**Done when:** the three views render live state and degrade gracefully when a stream drops.

## Phase 11 — Solana Migration (Real Blockchain Backbone)
**Owner:** B · **Depends on:** 0, **5**, **6**, **8**, **9** · **Parallel-safe:** after those
land · replaces the mocked ledger/consensus/storage mechanics that Phases 5, 6, 8, and 9
left in place with real Solana devnet integration.

- [ ] Anchor workspace (`programs/bio_digital_defense/`) with Threat Registry and Genome
      Registry account types (PDAs)
- [ ] `submit_threat` instruction: create/update a Threat Registry PDA, increment
      `Confidence_Score`
- [ ] `commit_gene` instruction: 3-of-5 multisig-gated write to a Genome Registry PDA — this
      is Proof of Immunity
- [ ] `suppress_gene` instruction: same multisig authority flips `Epigenetic_Status` to 1
- [ ] Deploy the program to Solana devnet; record the program ID in `Anchor.toml`
- [ ] Rust core: replace `ledger/client.rs`'s mock transport with `solana-client`/
      `solana-sdk` RPC calls
- [ ] Rust core: `ledger/state.rs` becomes commitment-level account reads (drop the custom
      Merkle-path code)
- [ ] `ledger/registry.rs` becomes a local read-through cache of the on-chain PDAs
- [ ] Devnet keypair provisioning per endpoint (Scout signer) and per Lymph Node validator
      (multisig signer), funded via faucet
- [ ] Gene binaries: real IPFS pinning-service integration (upload on `commit_gene`, fetch
      by CID before Soldier execution)
- [ ] Re-run `suppression-path-test` and `sandbox-isolation-check` against the live Solana
      path

**Done when:** a real devnet transaction commits a gene to the Genome Registry PDA under
multisig, a Soldier fetches and verifies it via RPC + real IPFS CID, and a `suppress_gene`
transaction halts it within seconds — no mocks left in the ledger path.

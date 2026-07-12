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

- [x] Anchor workspace (`programs/bio_digital_defense/`) with Threat Registry and Genome
      Registry account types (PDAs)
- [x] `submit_threat` instruction: create/update a Threat Registry PDA, increment
      `Confidence_Score`
- [x] `commit_gene` instruction: 3-of-5 multisig-gated write to a Genome Registry PDA — this
      is Proof of Immunity
- [x] `suppress_gene` instruction: same multisig authority flips `Epigenetic_Status` to 1
- [x] Deploy the program to Solana devnet; record the program ID in `Anchor.toml`
- [x] Rust core: replace `ledger/client.rs`'s mock transport with `anchor-client` RPC calls
      (chosen over hand-rolled `solana-client`/`solana-sdk` — builds instructions from the
      IDL; see `tech-stack.md`)
- [x] Rust core: `ledger/state.rs` becomes commitment-level account reads (dropped the
      custom Merkle-path code entirely)
- [x] `ledger/registry.rs` becomes a local read-through cache of the on-chain PDAs
- [x] Devnet keypair provisioning per endpoint (Scout signer, reusing the deployer wallet)
      and per Lymph Node validator (multisig co-signer — no funding needed, pure signers)
- [x] Gene binaries: real IPFS pinning-service integration (upload on `commit_gene`, fetch
      by CID before Soldier execution) — code in `ledger/ipfs.rs` targets Pinata, but is
      untested end-to-end pending a `PINATA_JWT` API key
- [x] Re-run `suppression-path-test` and `sandbox-isolation-check` against the live Solana
      path

**Done when:** a real devnet transaction commits a gene to the Genome Registry PDA under
multisig, a Soldier fetches and verifies it via RPC + real IPFS CID, and a `suppress_gene`
transaction halts it within seconds — no mocks left in the ledger path.

## Phase 12 — Live-Path Integration (closing the gaps)
**Owner:** all · **Depends on:** everything above · runs after Phases 0–11 land

Audit (2026-07-11): every box above is checked, but several phases landed as standalone
modules proven only by their own tests — the *live* demo path (`agents/scout.rs::spawn_demo`)
skips them, and the dashboard never receives a real event. These are the remaining gaps
between "each phase works in isolation" and "the program is fully implemented."

- [x] **Dashboard gets real events.** No `emit` exists anywhere in `src-tauri/` — inside the
      Tauri app all three views sit in their no-data state; only the browser mock generators
      are demoable. Plumb an `AppHandle` from `main.rs` into the Scout/Soldier/ledger paths
      and emit the `ecosystem`, `ledger`, and `strains` events `useTauriEvents` already
      listens for. Landed as `dashboard.rs`, threaded through as `Option<Arc<Dashboard>>`.
- [x] **Evolution runs in the live wake path.** `spawn_demo` pre-seeds a hardcoded
      `Allele04 + Allele12` gene; `evolution/` (Stage-2 fuzz driver, Stage-3 Lymph Node) is
      never invoked outside tests. On `PharmacyOutcome::NoCureAvailable`, the Soldier should
      snapshot the frozen target → fuzz in-sandbox → allergy-check → `commit_gene` under
      multisig → re-dispense. Landed as `agents::soldier::evolve_and_commit`.
- [x] **Happy-path demo before the kill-switch demo.** The seed commits then immediately
      suppresses, so the only live outcome ever shown is `Suppressed`. Show a cure being
      dispensed and `Neutralized` first, *then* suppress and re-wake to show the halt.
      `spawn_demo` now runs two waves against the same `Threat_ID`: wave 1 evolves live and
      shows `Neutralized`; only then does `suppress_gene` fire before wave 2 shows `Suppressed`.
- [x] **Confidence-threshold mobilization.** The on-chain `Confidence_Score` increments via
      `submit_threat`, but nothing reads it back — the Scout wakes the local Soldier
      directly, so Phase 2's "network-wide mobilization on threshold cross" never fires in
      the live path. Both demo waves report the same `Threat_ID`, so wave 2's `submit_threat`
      crosses `MOBILIZATION_THRESHOLD`; `spawn_demo`'s threat-report loop reads it back and
      logs `threat.mobilized`. Doesn't gate the local Stage-1 wake — that stays separate,
      per source-of-truth.md.
- [x] **Remove IPFS; store the gene bytes on-chain (over-engineering cut).** A gene is a
      `Vec<Allele>` over a 3-variant enum — a handful of bytes, smaller than the 64-char CID
      that points at it. The off-chain blob store (Pinata/`PINATA_JWT`, `reqwest`,
      `ledger/ipfs.rs`) is unjustified at this size. Replaced `GenomeEntry.ipfs_cid: String`
      with the gene's own bytes (`gene_seq: Vec<u8>`), dropped the fetch-and-verify step (the
      gene now comes straight from the trusted confirmed-commitment account read), and deleted
      `ipfs.rs` + the `reqwest`/Pinata dependency. Kept `gene_hash` as the registry's gene
      identity. Redeployed to devnet under a fresh program ID (`5r6fERX6CtJ8RHzpf5wLtY1SfptV8rk4JEHfu6yh5WZV`
      — the previously-recorded ID's upgrade authority wasn't available on this machine).
- [x] **One green pass of the ignored live tests** (`cargo test -- --ignored`): the devnet
      `submit_threat`/`commit_gene`/`suppress_gene` round trip is green against the freshly
      deployed program. Caught and fixed a pre-existing Phase-11 bug running this for the
      first time: `LYMPH_NODE_VALIDATORS` in `constants.rs` never actually matched the
      `keys/lymph-nodes/validator-*.json` keypairs the Rust core signs with, so no real
      multisig could ever clear `POI_QUORUM` — resynced the hardcoded pubkeys to the real
      keys and redeployed.
      The two real-process-suspension tests (`scripted_target_is_really_suspended`,
      `wake_releases_real_target`) don't pass *in this sandbox*: spawned helper processes get
      PIDs under 1000 here, tripping the `PID_FLOOR` guard in `scout::suspend`/
      `soldier::release_target` that's deliberately there to protect real low-numbered
      *system* PIDs (kernel/init) on a normal host — working as designed, just not
      exercisable in a container with an unusually low PID counter. Not weakening that guard
      to force a pass; these two need a normal dev machine to actually verify.

Deliberately still simulated (in scope per the preamble — only the malware itself and the
sandbox host stay fake): the scripted target as the "virus," the mock host inside
`evolution/sandbox.rs`, and the Lymph Node's mock Top-5,000 subset. Windows suspension
(`NtSuspendProcess`) stays a stub — the PoC targets macOS/Linux.

**Done when:** launching the app shows, live in the dashboard: the scripted target crossing
100 pts → Soldier wake → fuzz-discovered gene committed on devnet (gene bytes in the account,
no IPFS) → `Neutralized`, then a `suppress_gene` transaction halting the next wake — no manual
seeding.

## Phase 14 — Frontend ↔ Backend Live Link
**Owner:** C (+B for the Rust side) · **Depends on:** 12 · start anytime

Phase 12 landed the backend half (`dashboard.rs` emits real `ecosystem`/`ledger`/`strains`
events), but inside the running app the UI still shows mocks. Root causes found by audit
(2026-07-11): `useTauriEvents.js` detects Tauri via `"__TAURI__" in window`, which is only
injected under the unset `withGlobalTauri` config flag — so the hook always falls back to
its mock generators, even in-app; emits are edge-triggered and the demo is finite, so a
user who logs in and navigates after the two waves sees "Waiting for…" forever; and the
`strains` stream has no mounted consumer. Goal: `make dev` → signed-in Dashboard shows real
backend data, no mocks on the live path.

- [ ] **Fix Tauri detection** (`landing/src/hooks/useTauriEvents.js`): check
      `"__TAURI_INTERNALS__" in window` (always injected by Tauri 2) instead of
      `"__TAURI__"`. Keep the mock fallback for plain browser `vite dev`.
- [ ] **Snapshot re-emit loop** (`dashboard.rs` + `scout.rs`): `spawn_reemit(self: &Arc<Self>)`
      re-emits current `ecosystem`/`ledger`/`strains` state every ~2s; called from
      `spawn_demo`. Fixes late subscribers and the post-demo dead air / false `dropped`.
- [ ] **Real system stats — new `stats` stream**: add `sysinfo`; the re-emit thread also
      emits `{ ramMb, cpuPct, diskUsedGb, diskTotalGb, uptimeSecs, scouts, monitored, slot }`.
      `scouts` = AtomicUsize on `Dashboard` bumped by `Scout::spawn`; `slot` = real devnet
      slot via an `RpcClient::get_slot` closure passed from `spawn_demo` (None on failure).
      `Dashboard.jsx` uses real values when present, keeps the random-walk as no-data
      fallback. Delete the GPU card (no cross-platform source — deletion over a fake number).
- [ ] **Mount the `strains` stream**: add the existing `StrainTree.jsx` as a card in
      `pages/Dashboard.jsx` next to "Immune ledger". No double-mount of
      EcosystemGraph/LedgerTerminal — Dashboard already renders those streams its own way.
- [ ] **Real Protection page** (`pages/Protection.jsx`): derive the kill log from
      `threat.neutralized`, allergies from `gene.allergy_flagged`, suppressions from
      `gene.suppressed` via `useTauriEvents("ledger")`; summary tiles computed from the
      events; columns shrink to what's real (Threat ID, time, outcome — no invented scout
      names/TTK). Delete the static `KILLS` placeholders; empty state per degrade-gracefully.
- [ ] **Event copy for real kinds** (`pages/Dashboard.jsx`): extend `LEDGER_EVENT_COPY` /
      `LEDGER_BLOCK_STATUS` with `threat.mobilized`, `gene.fuzzed`, `gene.allergy_flagged`,
      `gene.suppressed`, `threat.neutralized`, `gene.ineffective`, `cure.unavailable`,
      `ledger.unavailable`; drop the mock-era `gene.proposed`/`poi.verified` entries. Fix the
      "Gene vector" line — IPFS was removed in Phase 12; show the threat hash instead.

Stays simulated, declared with `ponytail:` comments: active nodes, cures/min, soldier spore
counts, battery — network-wide fiction a single endpoint can't know.

**Done when:** `make dev` → log in → Dashboard shows the real `virus`/`disease` PIDs climbing
to `suspended`, the real ledger event sequence through `threat.neutralized` then
`gene.suppressed`, a growing StrainTree, and real RAM/CPU/scouts/slot numbers; navigating
away and back after the demo still renders (re-emit); Protection lists the real kills; plain
browser `vite dev` still falls back to mocks.

## Phase 13 — Beyond the Demo (path to a real implementation)

**Owner:** unassigned · **Depends on:** 0–12 · not required for the PoC bar (`CLAUDE.md`:
"it compiles and the demo works") — this is the gap between that bar and a system you'd
trust with real endpoints and real adversaries. Ordered by importance: fixing #1–2 changes
whether this is a real antivirus that happens to use a blockchain; everything below them is
real infrastructure wrapped around however real the top of the list is.

1. **~~Real Scout / behavioral detection substrate~~ — done for 2 of 3 actions.**
   `RealBehaviorSource` (`agents/scout.rs`) replaces `ScriptedSource`, spawning real
   `fake_viruses/virus`/`disease` processes and detecting their actual behavior via `lsof`
   (fd-count burst → `HighEntropyFileLoop`; live connection → `NetEnumWithVssTamper`),
   verified against real specimens with `#[ignore]`d tests
   (`disease_specimen_is_really_detected_and_woken`,
   `virus_specimen_is_really_detected_and_woken`). A real engineering problem surfaced and
   got fixed along the way: at every polling rate tried (500ms down to bash-loop-max-speed
   ~48ms), `lsof` almost never caught either specimen's original suspicious window (`disease`
   2/41 hits; `virus` never once above baseline) — `lsof` itself takes ~40-50ms/call, faster
   than the specimens' original single-digit-millisecond windows. Fixed by widening the
   specimens' own windows (`kHoldOpenDuration = 200ms` in both `.cpp` files) rather than
   switching to a less-honest detection signal (parsing the specimens' own stdout was
   considered and rejected — it would've just been `ScriptedSource`'s cooperative-signal
   problem again, dressed up). `Action::HiddenChildFromTemp` (`bacteria`) still has **no
   real detector** — deliberately deferred, not faked: `bacteria` re-forks itself with no
   `exec()`, so there's no Temp-directory path to check against the literal spec wording;
   revisit when a real temp-dir-dropper specimen exists. No kernel-level tracer (eBPF/
   ETW/EndpointSecurity) — each is blocked for this project specifically, not just costly
   (macOS's EndpointSecurity needs an Apple entitlement granted only to vetted security
   vendors), not just deprioritized effort.
2. **~~Real sandbox / execution substrate~~ — isolation mechanism done; attack content
   deliberately stays synthetic.** `Sandbox::run()` (`evolution/sandbox.rs`) now executes a
   real `wasmi` Wasm module (`EVALUATE_WAT`, a hand-written WAT constant) instead of a
   hardcoded Rust `match` — genuine isolated execution, verified by a
   `compiled_gene_module_has_zero_imports` test asserting the module declares no host
   imports at all, so it is provably incapable of touching the real host, memory, files, or
   network. The compiled `Module` is built once per `Sandbox` and reused across calls; only
   the `Store`/`Instance` execution state is fresh per `run()`, matching genuine per-trial
   isolation without wastefully re-parsing static WAT on every fuzz combo. The
   attack-effectiveness decision inside the module is unchanged synthetic physics
   (`Allele09` destabilizes, `Allele04`+`Allele12` crashes) — deliberately **not** made real,
   both because building actual working exploit primitives is out of scope for a hackathon
   PoC without separate explicit authorization, and because there's no real cross-process
   target memory yet to decide over (`FrozenProcess.memory` stays mocked — real memory
   cloning needs privileged `ptrace`/`task_for_pid`, blocked by the same kind of platform
   entitlement story that blocked kernel tracers in item #1). All 3 pre-existing physics
   tests pass unchanged against the real execution path, confirming no behavior regression.
3. **Real gene payload.** Follows directly from #2 — a gene is `Vec<Allele>` over a
   3-variant enum (`gene_seq: Vec<u8>`, max 3 bytes on-chain), not compiled bytecode. Once
   the sandbox is real, the "Wasm Gene Payload" needs to become an actual executable
   artifact, which reopens the on-chain-storage-size question Phase 12 closed by removing
   IPFS (a real payload won't fit in an account the way 3 bytes does).
4. **Real decentralization of the Lymph Node multisig.** All 5 `LYMPH_NODE_VALIDATORS`
   keypairs are held and signed by one process (`spawn_demo`) — "3-of-5 agreed" currently
   means one process decided to sign 5 times, not 5 independent validators independently
   agreeing. The validator set is also hardcoded in `constants.rs` with no governance,
   rotation, or on-chain registry — and it already drifted out of sync with the actual
   keypair files once this session, caught only by chance when a live test happened to run
   (see Phase 12's note). Needs: actually-separate validator services, each running its own
   regression check before signing, plus a way to add/rotate/verify the validator set that
   isn't hand-copying pubkeys into a Rust source file.
5. **Real Lymph Node regression corpus.** `TOP_APPS` is 4 hardcoded entries with a static
   `allergic_to` lookup, not "Top 5,000 apps" actually executed and observed for
   crash/leak. Needs a real corpus and a real execution/observation step, not a match
   statement.
6. **Threat Registry schema persistence + a real mobilization subscriber.** Only
   `behavioral_schema_hash` reaches the chain — the raw `Behavioral_Schema` a Scout
   observed exists only in that process's memory for one report's lifetime, so a second
   endpoint with a matching hash has no way to retrieve *what the schema was*. Separately,
   `ThreatMobilized` fires and gets logged but nothing *acts* on it — no other endpoint is
   notified, no automated response starts.
7. **Multi-endpoint reality.** Everything above assumes one endpoint. The "collective
   memory" and "crowd-sourced immunity" framing needs an actual network of independent
   Scouts whose sightings genuinely correlate — today `MOBILIZATION_THRESHOLD` is only ever
   crossed by the same one process reporting twice.
8. **Kill-switch automation + a governed reactivation path.** Suppression only happens
   because a human calls `suppress_gene` — no monitoring/alerting layer proposes it
   automatically. Separately, `commit_gene`'s guard permanently refuses to reactivate a
   suppressed gene (deliberate, for the demo's safety story); a real system likely wants a
   reactivation path gated by its own multisig for a wrongly-flagged allergy, rather than a
   permanently dead gene.
9. **Windows support.** `suspend()`/`release_target()` are `#[cfg(unix)]` with a no-op
   stub elsewhere — no `NtSuspendProcess`-based suspension despite the CI matrix building
   for `windows-latest`. Ranked low deliberately: this is platform breadth, not core
   function — adding it doesn't change whether the system detects or cures anything real.
10. **Production ops hardening.** Devnet only, no mainnet posture; single public RPC
    endpoint with no fallback; program upgrade authority is a single wallet with no
    multisig/timelock (and the currently-deployed program's authority doesn't match the
    locally-recorded deployer wallet — worth reconciling); all keypairs are plain local
    JSON files with no HSM/secrets-manager story; CI builds `src-tauri` across 3 OSes but
    doesn't appear to build or test `programs/bio_digital_defense`, so an on-chain-program
    regression wouldn't fail CI the way a Rust-core one would.

**Done when:** items 1–3 land and the demo's "detection → evolved cure" claim is true
against a real (even if simple) syscall observer and a real (even if minimal) execution
sandbox, rather than a script and a lookup table.

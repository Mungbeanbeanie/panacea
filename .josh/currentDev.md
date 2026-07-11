# Current Dev Plan — Phase 11 (Solana Migration)

> Scratch planning doc only (`.josh/` is gitignored). Conforms to
> `.claude/docs/source-of-truth.md` (canonical) and `.claude/docs/plan.md` Phase 11
> (owner B).
>
> **✅ Boxes 1–9 and 11 implemented and verified against real Solana devnet.** Program:
> `FPn8ftGPZ5fr6rH3qtttBp2ppx7HNV4zscGhkFSv8t5x`. A live (`#[ignore]`d) test —
> `ledger::client::live_devnet_tests::submit_commit_and_suppress_round_trip_against_devnet`
> — proved `submit_threat` → `ThreatRegistry` read, 3-of-5-multisig `commit_gene` →
> `GenomeRegistry` read, and `suppress_gene` → a **freshly constructed** `GenomeRegistry`
> correctly reading `epigenetic_status = 1` back (ruling out stale in-memory cache), all
> against the real deployed program. Cost ~0.003 SOL total. Offline suite: 16 passed, 0
> failed, 2 ignored (pre-existing real-process tests, unaffected).
>
> **🟡 Box 10 (real IPFS) blocked on your Pinata API key.** `ledger/ipfs.rs` is fully
> written (Pinata `pinJSONToIPFS`/gateway fetch, `PINATA_JWT` env var) and compiles, but
> is untested end-to-end — needs your `PINATA_JWT` to actually run. Everything else in
> Phase 11 is done.

## Scope (`.claude/docs/plan.md` Phase 11)

| # | Phase 11 box | Status |
|---|---|---|
| 1–5 | Anchor program, all 3 instructions, devnet deploy | ✅ done |
| 6 | `ledger/client.rs`: real RPC transport (`anchor-client`) | ✅ done, live-verified |
| 7 | `ledger/state.rs`: commitment-level reads, drop Merkle code | ✅ done, live-verified |
| 8 | `ledger/registry.rs`: read-through cache of on-chain PDAs | ✅ done, live-verified |
| 9 | Devnet keypair provisioning (endpoints + validators) | ✅ done, live-verified |
| 10 | Real IPFS pinning-service integration | 🟡 code written, blocked on `PINATA_JWT` |
| 11 | Re-run `suppression-path-test` / `sandbox-isolation-check` against the live path | ✅ done — see below |

## What already exists (the mock boxes 6–11 replace — read this session)

- **`src-tauri/src/ledger/registry.rs`** — `ThreatRegistry` (`report()`/`get()`,
  `MOBILIZATION_THRESHOLD = 2`) and `GenomeRegistry` (`publish()`/`get()`/`suppress()`),
  both in-memory `HashMap<ThreatId, _>`, plus `MockIpfsStore`.
- **`src-tauri/src/ledger/client.rs`** — `MockConjugationLink`: P2P stand-in with
  `commit_block`, `broadcast_suppressor`/`drain_suppressors`, `threat_proof`/`genome_proof`.
- **`src-tauri/src/ledger/state.rs`** — `StateLedger`/`MerkleTree`/`MerkleProof`/
  `BlockHeader`/`Hash`: custom Merkle-path construction and verification.
- **`src-tauri/src/agents/soldier.rs`** — `Pharmacy { genome, ipfs, state, link }` and
  `resolve_and_run(threat_id, &GenomeRegistry, &MockIpfsStore, &StateLedger, &MerkleProof,
  Sandbox)`, both built directly against the mock types above.
- **`src-tauri/src/agents/scout.rs`** — `spawn_demo()` constructs a `Pharmacy` directly the
  same way.
- **`src-tauri/src/evolution/lymph_node.rs`** — confirmed untouched by this slice; Stage 3's
  allergy check is off-chain and independent of the ledger.
- **`src-tauri/Cargo.toml`** — no `solana-client`/`anchor-client`/HTTP crate yet; all new
  dependencies for this slice.

## The cross-cutting fork this slice can't avoid (read first)

Boxes 6–8 are listed as `ledger/`-only (owner B) in plan.md, but they aren't self-contained.
`agents/soldier.rs`'s `Pharmacy` and `resolve_and_run` are built directly against today's
mock types. Once those types become Solana-backed, `resolve_and_run`'s signature
necessarily changes too — most importantly, **the `MerkleProof` parameter disappears
entirely**, because `source-of-truth.md`'s Ledger 1 section now says reading a Solana
account at a trusted commitment level *already* carries the integrity guarantee a Merkle
path used to provide. `agents/scout.rs`'s `spawn_demo()` needs the same update.

This is real owner-A/owner-B boundary crossing, the same shape as the Gap-B issue flagged
back in the Phase 9 planning history. Flagging it up front: **touching `agents/soldier.rs`
and `agents/scout.rs` is unavoidable to finish this phase**, not scope creep — the plan's
own done-when criterion ("a Soldier fetches and verifies it via RPC + real IPFS CID")
requires it.

## New dependencies — unverified, don't guess versions at implementation time (Rule 4)

None of these are in `src-tauri/Cargo.toml` yet:

- **Decided: `anchor-client`** (Q1) over hand-rolled `solana-client` + manual instruction
  encoding — builds instructions from the IDL rather than hand-encoding them, the natural
  complement to the program we already built with Anchor. Its exact version compatible
  with `anchor-lang 1.1.2` and the Solana "v3" crate family (`solana-pubkey v3.0.0`,
  `solana-transaction v3.4.0`, etc.) is still unverified — check by actually resolving it
  at implementation time, don't assert a version from memory.
- An HTTP client for the IPFS pinning service — `reqwest` is the standard choice, version
  unverified against this project's edition/toolchain.
- **Decided: stay blocking/synchronous, not async** (Q2). The whole agent core today is
  `std::thread` + `mpsc` — no `tokio` anywhere in `src-tauri`. The async (`nonblocking`)
  Solana client would ripple `async`/`.await` through `Pharmacy`, `resolve_and_run`,
  `Soldier::dispense`, `Scout::tick`, and `main.rs` — a much bigger refactor than this
  phase needs. The blocking client keeps the change contained to `ledger/` +
  `agents/soldier.rs`/`scout.rs`'s call sites (Rule 2/3).
- **Doc-sync reminder:** `plan.md`'s box 6 wording literally names "solana-client/
  solana-sdk"; since `anchor-client` was chosen instead (Q1), update that wording and
  `tech-stack.md`'s matching line once implemented, rather than letting the docs drift.

## Box 6 — `ledger/client.rs`: real RPC transport

Replaces `MockConjugationLink` with a thin wrapper — tentatively `SolanaConjugationLink` —
around the blocking RPC client, exposing:

- `submit_threat(threat_id, schema_hash) -> Signature` — signs with the caller's keypair,
  sends the `submit_threat` instruction.
- `commit_gene(threat_id, gene_hash, ipfs_cid, validator_keypairs: &[Keypair]) -> Signature`
  — the multisig call. **Decided (Q4):** the demo process holds all 5 Lymph Node keypair
  files rather than standing up 5 independent validator processes; it loads 3+ of them to
  co-sign a single transaction, simulating "quorum was reached." Matches the PoC bar;
  proves the on-chain multisig gate genuinely works, doesn't demonstrate real
  decentralization — accepted tradeoff.
- `suppress_gene(threat_id, validator_keypairs: &[Keypair]) -> Signature` — same shape.

No more `commit_block`/Merkle leaves — a Solana transaction's confirmation *is* the commit.

## Box 7 — `ledger/state.rs`: commitment-level reads, drop the Merkle code

Deletes `MerkleTree`, `MerkleProof`, `Side`, `BlockHeader`, `Hash`, `StateLedger`, and their
tests wholesale — plan.md's box 7 says so explicitly, and nothing else in the codebase
constructs a Merkle proof outside this file and its own tests. Replaces it with something
like a `SolanaLightClient` wrapping the RPC client:

- `get_threat_entry(threat_id) -> Option<ThreatEntry>` (deserialized on-chain account)
- `get_genome_entry(threat_id) -> Option<GenomeEntry>`

Both reads at a chosen commitment level — **decided: `confirmed`** (Q3).
`source-of-truth.md` permits *either* ("at `confirmed` or `finalized` commitment") — not a
spec-compliance question, purely a tradeoff, and `confirmed` was chosen for demo
responsiveness over `finalized`'s stronger-but-slower guarantee.

This is the biggest deletion of this slice — flagging clearly rather than doing it
silently, per Rule 3.

## Box 8 — `ledger/registry.rs`: read-through cache of on-chain PDAs

`ThreatRegistry`/`GenomeRegistry`'s `HashMap`-backed storage goes away; they become thin
wrappers that call the box-7 light client on every `get()`/`report()`/`publish()`-equivalent
call rather than holding local state — "cache" here means "the read path," not a persistent
local store, though a short-lived in-memory memo (avoid two RPC round-trips for the same
`threat_id` within one Soldier dispense) is a reasonable, small addition — plan.md's own
box wording calls this a "cache," so this isn't scope creep. `MockIpfsStore` also currently
lives in this file — **decided (Q6): it moves to a new `ledger/ipfs.rs`** for
single-responsibility, since box 10 turns it from a trivial `HashMap` wrapper into a real
HTTP-client integration, a genuinely distinct concern from this file's job. Add to
`file-structure.md` when implemented.

## Box 9 — Keypair provisioning

- **Lymph Node validators:** already done (5 real keypairs). **Correction on funding:**
  re-checked the actual `commit_gene`/`suppress_gene` account structs — `validator_1..5`
  are plain `Option<Signer<'info>>` with no `#[account(mut)]`; they're pure co-signers that
  never pay rent or fees. Producing a signature costs nothing regardless of balance — only
  the designated `payer` needs SOL. plan.md's box 9 "funded via faucet" wording really only
  applies to the endpoint/Scout signer below; funding the 5 validator wallets would be
  wasted faucet requests. Nothing new needed here.
- **Endpoint/Scout signer:** plan.md calls for "per endpoint." **Decided (Q5):** reuse the
  existing deployer wallet as the Scout's signer too, rather than generating a second
  keypair that would also need its own faucet funding — simplest option consistent with
  "one demo node."
- **Funding headroom:** each instruction call is a tiny tx fee (~0.000005 SOL) plus, for a
  **new** PDA, a small rent-exemption (far less than the program deploy's 1.32 SOL).
  Deployer wallet's remaining ~1.16 SOL should comfortably cover the demo's transactions.

## Box 10 — Real IPFS pinning-service integration

- New `ledger/ipfs.rs` (see box 8), wrapping **Pinata's** HTTP API (Q9) via `reqwest`:
  `upload(bytes) -> Cid` (called from the `commit_gene` client flow) and
  `fetch(cid) -> bytes` (called from the Soldier's dispense path, replacing
  `MockIpfsStore::fetch`).
- **Real external blocker, not something I can resolve myself:** needs an actual Pinata
  account + API key — creating that account is yours to do. Same shape as the devnet
  faucet blocker earlier in this phase.
- The API key goes in `.env` (already gitignored), read via **`std::env::var` directly, no
  new dependency** (Q7) — documented in `build-run-test.md` as a setup step rather than
  auto-loaded by a `dotenvy`-style crate.
- After fetch, the Soldier compares `sha256(fetched_bytes) == entry.gene_hash` directly —
  this *is* the verification step now (see box 7); no separate proof object.

## Box 11 — Re-run the safety skills against the live path

`agents/soldier.rs`'s `pharmacy_flow_tests` and `live_dispense_tests` currently construct
the mock `Pharmacy` directly and will break once boxes 6–8 change its field types.
**Decided (Q8): a `GenomeSource`-style trait boundary** that `resolve_and_run` depends on,
so the existing tests keep running fast and offline against a fake implementation, with a
small number of new `#[ignore]`d live-devnet tests covering the real path (mirrors the
existing pattern already used for the real-process tests in `soldier.rs`/`scout.rs`) — over
converting the existing tests to hit live devnet directly (would have been simpler, but
slower, costs real devnet SOL per test run, and flaky under network conditions) and over an
internal `Backing::Rpc(...) | Backing::Offline(HashMap<...>)` enum on the concrete types
(would have avoided a generic/dyn-dispatch boundary entirely, at the cost of a branch
inside each method instead of a clean seam).

Once implemented, re-run `suppression-path-test` and
`sandbox-isolation-check` per the box's own wording. Worth noting `sandbox-isolation-check`
specifically: boxes 6–10 never touch `evolution/sandbox.rs` or the gene-execution call
(`sandbox.run(&gene.sequence)` is unchanged — only *where the bytes come from* changes), so
that check should trivially still pass. Re-running it is compliance with the box's literal
wording, not a response to any new risk this slice introduces.

## Open questions — all resolved

1. ~~RPC client choice~~ — **`anchor-client`.**
2. ~~Blocking vs. async RPC~~ — **stay synchronous** (blocking `RpcClient`, no `tokio`).
3. ~~Commitment level~~ — **`confirmed`** (both `confirmed`/`finalized` are equally
   on-spec; chosen for demo responsiveness).
4. ~~Multisig demo simplification~~ — **one process holds all 5 Lymph Node keypairs**
   and self-signs 3+ onto each transaction, rather than standing up 5 separate validator
   processes. Matches the PoC bar; proves the on-chain multisig gate genuinely works, does
   not demonstrate real decentralization — that tradeoff is accepted.
5. ~~Scout signer~~ — **reuse the deployer wallet** (`~/.config/solana/id.json`,
   `HeAubH3AUZDwztNC3BCsDSacnGSAXnd2ZpLJ3H68W6b3`) as the Scout's signer too. No new
   keypair, no new faucet round.
6. ~~`ledger/ipfs.rs` as a new file~~ — **yes, split it out.** Concrete justification:
   `MockIpfsStore` today is a trivial `HashMap` wrapper, but box 10 turns it into a real
   HTTP-client integration (`reqwest`, API-key auth, CID handling) — a genuinely distinct
   concern from `registry.rs`'s job (mirroring Solana account reads), and a concrete,
   present change rather than a hypothetical one. Add to `file-structure.md` when
   implemented.
7. ~~`.env` loading~~ — **`std::env::var` directly, no new dependency.**
8. ~~Test strategy (box 11)~~ — **`GenomeSource`-style trait**, existing tests stay
   offline against a fake implementation; a small number of new `#[ignore]`d live-devnet
   tests cover the real path (mirrors the existing pattern already used for the
   real-process tests in `soldier.rs`/`scout.rs`).
9. ~~Pinning service account~~ — **Pinata recommended** (real free tier as of 2026: 1GB /
   500 files; mature REST API, well-suited to a `reqwest` integration). web3.storage's old
   free/simple tier has been wound down in favor of an enterprise model, deprioritizing it;
   Filebase is a viable paid-from-byte-one alternative if Pinata's limits are ever an
   issue. **You still need to create the Pinata account and hand me the API key** — that
   account creation isn't something I can do on your behalf.

## Recommendation

All open questions are resolved. Remaining before "implement": you creating the Pinata
account and providing the API key (box 10's external blocker) — everything else (boxes
6–9, 11) has no external dependency left and can proceed as soon as you say "implement."

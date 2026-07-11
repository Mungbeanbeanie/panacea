# Current Dev Plan — Phase 9 (Epigenetic Suppression Kill-Switch)

> Scratch planning doc only (`.josh/` is gitignored). Nothing here is implemented until the
> user says **"implement"**. Conforms to `.claude/docs/source-of-truth.md` (canonical).
> **First-class safety** — keep `suppression-path-test` passing.
>
> **⚠️ Rewritten after `git pull` (HEAD `6d0f325`).** The teammate landed **Phase 6**
> (`a99ae3f`) and **Phase 7** (`6d0f325`). Phase 6 already implemented the kill-switch core,
> so most of this phase is **done**. What follows is the corrected, much smaller remainder.

## Scope (`.claude/docs/plan.md:157–160`) — status after the pull

| # | Phase 9 box | Status |
|---|---|---|
| 1 | Epigenetic Suppressor Token **broadcast** → set `Epigenetic_Status = 1` | **Partial** — flag-set done; broadcast/receipt missing |
| 2 | Soldiers reading Ledger 3 immediately stop executing that cure | ✅ **Done** |
| 3 | Status check happens **before** any gene fetch/exec | ✅ **Done** |
| 4 | Keep `suppression-path-test` passing | ✅ **Done** (passing) |

**Done when:** flipping `Epigenetic_Status` to 1 halts the cure in seconds, before any
fetch/exec. — *Logic proven; not yet demonstrable through the live daemon (see Gap B).*

## What already exists (verified this session)

In `agents/soldier.rs`, the teammate's `resolve_and_run(threat_id, genome, ipfs, state_ledger,
gene_proof, sandbox) -> PharmacyOutcome` **is the kill-switch**:

```
get(threat_id)                        → NoCureAvailable if absent
  └─ if Epigenetic_Status == Suppressed → return Suppressed   ← FIRST, before verify/fetch/run
       (else) verify_gene against Merkle root → GeneHashUnverified if bad
       fetch bytecode from MockIpfsStore
       sandbox.run(&gene.sequence); sandbox.teardown()  ← in-sandbox only, then apoptosis
```

- **Order is correct:** suppression is the first branch after lookup — before Merkle verify,
  before `ipfs.fetch`, before `Sandbox::run`. ✅ (matches `suppression-path-test` + `security.md`)
- **Test already passing:** `pharmacy_flow_tests::suppressed_gene_is_halted_before_any_fetch`
  asserts `PharmacyOutcome::Suppressed` **and `ipfs.fetch_calls() == 0`** — I ran it: **ok**
  (all 4 pharmacy-flow tests green).
- **Flag setter:** `ledger::registry::GenomeRegistry::suppress(&threat_id)` flips
  `Active → Suppressed`.
- **Transport available for a real broadcast:** `ledger::client::MockConjugationLink`
  (commit/fetch-header/Merkle-path). No suppressor-token type rides it yet.

## What actually remains

### Gap A — Step 1's "broadcast" (owner-B territory)
`suppress()` is a **local method call**; there is no *Epigenetic Suppressor Token* that is
broadcast over the (mock) conjugation transport and applied on receipt. To satisfy step 1 as
written, add a thin token + apply path, e.g.:
- a `SuppressorToken { threat_id }` (or gene id) type,
- `MockConjugationLink::broadcast_suppressor(token)` / a receive hook that calls
  `genome.suppress(&token.threat_id)` on the local registry.

This lives in **`ledger/` (owner B)** — `registry.rs` + `client.rs`. **Coordinate before
touching** (Rule 3 / collaboration). It may already be on the teammate's list since Phase 9 is
theirs.

### Gap B — Live wiring so the kill-switch is demonstrable end-to-end
`resolve_and_run` is currently called **only from its own tests**; the live Soldier daemon
(`handle_wake` → `soldier.neutralize()`, the Phase-3 mock) never invokes it. So "halts in
seconds" isn't observable in the running app. Closing this = wiring the pharmacy flow into the
live wake lifecycle:
- thread a shared `GenomeRegistry` + `MockIpfsStore` + `StateLedger` (+ a proof source) into
  `soldier::run`,
- replace `neutralize()` with `resolve_and_run(...)` in `handle_wake`,
- a runtime suppressor broadcast (Gap A) then halts the next wake's cure.

This is **Phase 6's Soldier-side integration as much as Phase 9's** — it crosses the
agents/ledger boundary and is a bigger, coordination-heavy change.

## ⚠️ Ownership question (needs your call before any implementation)

Phase 9 is **owner B**, and the teammate is actively in `soldier.rs`/`registry.rs`/`client.rs`
(they just implemented the core). Both remaining gaps sit largely in **their** files. Options:
- **Leave Phase 9 to owner B** (recommended) — the core is theirs and done; Gaps A/B are the
  natural continuation of their Phase 6 work. I stay out to avoid merge collisions.
- **I take Gap B's agents-side wiring only** (`soldier::run` / `scout::spawn_demo`, my files),
  coordinating the shared-state signature with B.
- **I take Gap A** (their `ledger/` files) — only with explicit go-ahead.

## Source-of-truth conformance (of what exists)
- Soldier queries Ledger 3 **on demand** by `Threat_ID` (`genome.get`), never a passive scan. ✅
- `Epigenetic_Status` checked **before** verify/fetch/exec; suppressed returns before
  `ipfs.fetch`/`Sandbox::run`. ✅
- Gene executes **only** in the sandbox (`Sandbox::run`), never the host. ✅
- `suppression-path-test` invariant satisfied and covered by a passing test. ✅

## Recommendation
Phase 9's safety-critical core is **already complete and passing** — no action needed to make
the kill-switch *correct*. The only open items are step 1's **broadcast** wrapper (Gap A) and
**live daemon wiring** (Gap B), both mostly in owner B's files. **Recommend: confirm with the
teammate before I touch anything here.** If you want a demonstrable live kill-switch, the
highest-value slice is Gap B's agents-side wiring — say so and I'll plan/implement just that.

## Open questions for the user
- **Q1:** Given the core is done + owner B's territory, do you want me to do anything on Phase 9
  at all, or leave it to your teammate?
- **Q2:** If yes — Gap A (broadcast, in `ledger/`) or Gap B (live wiring, spanning `agents/`)?
- **Q3:** For Gap B, OK to change `soldier::run`'s signature (shared registry/IPFS/state) and
  touch `scout::spawn_demo` again?

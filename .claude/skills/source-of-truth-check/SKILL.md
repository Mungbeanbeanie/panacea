---
name: source-of-truth-check
description: Self-check that a change conforms to the canonical Source of Truth (.claude/docs/source-of-truth.md) — correct thresholds, stage boundaries, ledger fields, and data flow. Run before finishing any task that touches a stage, ledger, or agent, so work doesn't drift off the intended architecture.
---

# Source-of-truth check

Run this before you call a task done. It's a fast alignment pass against the canonical spec
in [source-of-truth.md](../../docs/source-of-truth.md) — the goal is to catch drift before
it merges, not to re-review correctness.

## Procedure

1. **Locate the touched surface.** Name which part(s) of the spec your change affects — a
   pipeline stage (1–4), a ledger (1 State / 2 Threat / 3 Genome), a caste (Scout/Soldier),
   or the suppression path.
2. **Re-read that section** of `source-of-truth.md`.
3. **Check against the spec:**
   - **Thresholds & values** match (Stage 1: Action A +20, B +50, C +40; interrupt at
     **100** cumulative pts). Don't invent or round different numbers.
   - **Stage boundaries** hold — Scouts observe and score only (never kill); genes/alleles
     execute **only** inside the sandbox, never the live host.
   - **Ledger fields & data flow** match — Ledger 2 carries `Threat_ID` /
     `Behavioral_Schema` / `Confidence_Score`; Ledger 3 carries `Threat_ID → Wasm_Gene_Hash`
     / `IPFS_URI` / `Epigenetic_Status`. A Soldier queries the Genome Registry **on demand**
     by `Threat_ID` (never a passive scan), **verifies the hash against the State Ledger
     Merkle root** before use, and checks `Epigenetic_Status` **before** fetch/exec.
   - **UI stays observability-only** — no detection/remediation/consensus logic in `landing/`.
   - **No invented protocol/config/crate details** (Working Agreement, Rule 4) — verify
     against `Cargo.toml` / `package.json` / the spec, don't guess.
4. **On a contradiction, stop and flag** — don't silently diverge. If the *spec itself*
   needs to change, update `source-of-truth.md` first and say so.
5. **Emit a verdict:** `ON-SPEC` or `OFF-SPEC`, listing each divergence and where it is.

Keep it lightweight — this is a drift guard, not a full review. For safety-specific checks
use [`suppression-path-test`](../suppression-path-test/SKILL.md) and
[`sandbox-isolation-check`](../sandbox-isolation-check/SKILL.md).

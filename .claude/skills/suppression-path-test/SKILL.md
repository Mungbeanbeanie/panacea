---
name: suppression-path-test
description: Verify the epigenetic kill-switch — that a gene with Epigenetic_Status = 1 is halted before it is ever fetched or executed, and can't be trivially bypassed. Use whenever changing the Genome Registry (ledger/registry.rs) or Soldier execution (agents/soldier.rs).
---

# Suppression-path test

The kill-switch is the demo's emergency brake (see
[architecture.md](../../docs/architecture.md) and [security.md](../../docs/security.md)).
Keep this check passing whenever you touch the Genome Registry or Soldier execution.

## The invariant

A Soldier queries the Genome Registry by `Threat_ID`, reads `Epigenetic_Status`, and if it
is `1` (suppressed) it **refuses the cure before fetching the gene from IPFS or executing
it**. The check comes first — not after download, not after a partial run.

## Procedure

1. **Locate the execution path.** In `agents/soldier.rs`, find where a Soldier resolves a
   `Threat_ID` to a gene and runs it. Confirm the very first thing it does with the
   registry entry is check `Epigenetic_Status`.
2. **Suppressed → halt.** Arrange a registry entry with `Epigenetic_Status = 1`. Assert the
   Soldier returns/aborts *without* calling the IPFS fetch and *without* executing anything.
3. **Active → proceed.** With `Epigenetic_Status = 0`, assert the Soldier proceeds normally
   (into the sandbox path — never the live host).
4. **Order matters.** Confirm no code path fetches or executes the gene before the status
   check. If the fetch is mocked for the PoC, assert the mock is *not* invoked in the
   suppressed case.

## Keep one runnable check

Leave a small Rust test behind so a regression fails loudly, e.g.:

```rust
#[test]
fn suppressed_gene_never_executes() {
    let registry = registry_with_status(THREAT_ID, EpigeneticStatus::Suppressed);
    let outcome = Soldier::attempt_cure(THREAT_ID, &registry, &fetch_spy);
    assert!(matches!(outcome, CureOutcome::HaltedSuppressed));
    assert_eq!(fetch_spy.calls(), 0, "must not fetch a suppressed gene");
}
```

Adjust names to the real API (Working Agreement, Rule 4 — don't invent signatures; read
the code). The point is: suppressed halts, and the fetch spy shows zero calls.

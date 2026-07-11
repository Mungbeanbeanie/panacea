# Security & Safety (PoC-scoped)

This codebase suspends processes, runs isolation sandboxes, and distributes remediation
payloads. For the hackathon the goal is to **model these safety properties and show them
working on the happy path** — not to production-harden them. Two failure modes to avoid:
over-investing in hardening the demo doesn't need, and *faking* an invariant so the demo
lies about what it does. Keep the safety story honest and simple.

## Boundaries to keep

- **Exploit primitives (`alleles.rs`) and generated genes execute only inside the
  isolation layer in `sandbox.rs`.** Never add a code path that runs a gene or allele
  directly against the live host outside a verified sandbox. This is the one boundary
  worth guarding even in a PoC — a gene escaping the sandbox is the difference between a
  demo and an incident. See the
  [`sandbox-isolation-check`](../skills/sandbox-isolation-check/SKILL.md) skill.
- **Don't weaken isolation for convenience.** Sandbox escape, disabled seccomp/capability
  restrictions, or "temporary" host access aren't acceptable shortcuts. If a task seems to
  require it, stop and flag it (Working Agreement, Rule 1).
- **The suppression path must always work.** Any change touching the Genome Registry or
  Soldier execution preserves the guarantee that `Epigenetic_Status = 1` halts a cure
  immediately, checked *before* any gene fetch/exec. Keep the
  [`suppression-path-test`](../skills/suppression-path-test/SKILL.md) check passing.
- **Verify before trusting the network.** Every threat and gene fetched from peers or IPFS
  is untrusted until verified against the State Ledger Merkle root. In the PoC you may mock
  the network, but the verification step stays in the flow — don't skip it and pretend.
- **Don't commit real malware, live exploit chains, or captured host telemetry.** Use
  synthetic fixtures. If unsure whether something is safe to commit, don't — ask.
- **Least privilege in Tauri.** Grant capabilities in `tauri.conf.json` narrowly. Don't
  broaden permissions to make an error go away.

## When you're unsure — escalate, don't guess

Ask or flag explicitly (rather than silently deciding) when:

- A request is ambiguous about scope, a data shape, or which stage/ledger it touches.
- A change would cross a boundary above — UI gaining logic, weakening isolation, altering
  consensus or the suppression path.
- You'd need to invent a crate, API, config key, or protocol detail you can't verify.
- The "simple" fix implies a large refactor — surface the tradeoff and let a human choose.

A short, honest question now is cheaper than a confident wrong build later.

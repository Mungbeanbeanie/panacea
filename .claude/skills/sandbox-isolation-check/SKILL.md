---
name: sandbox-isolation-check
description: Confirm that alleles and generated genes execute only inside the isolation layer in evolution/sandbox.rs, never against the live host. Use when changing evolution/ or adding any gene/allele execution path.
---

# Sandbox-isolation check

The one boundary worth guarding even in the PoC: a gene escaping the sandbox is the
difference between a demo and an incident (see [security.md](../../docs/security.md)).

## The invariant

Every execution of an allele (exploit primitive) or a compiled gene happens **inside the
sandbox handle from `evolution/sandbox.rs`**. There is no code path that runs either
directly against the live host.

## Procedure

1. **Find every execution site.** Grep `evolution/` and `agents/soldier.rs` for where
   alleles are invoked and where a Wasm gene is run (the Wasm runtime's `call`/`invoke`,
   the fuzz driver in `alleles.rs`, the remediation step in `soldier.rs`).
2. **Trace each to the sandbox.** Confirm each site receives its execution context from a
   sandbox handle — memory is cloned into the MicroVM/Wasm container, the target is a mock
   host, and results come back out. No site takes a raw live-host handle.
3. **No isolation weakening.** Check the sandbox setup doesn't disable seccomp/capability
   restrictions or grant "temporary" host access to make something work.
4. **Teardown exists.** Every sandbox handle has a defined teardown path — no dangling
   containers or leaked cloned memory after apoptosis.

## Keep one runnable check

Leave a small test asserting the execution API requires a sandbox context — i.e. there is
no public function that runs a gene/allele without one. If a mock host is used, assert the
run never touches a real host handle. Adjust to the real API; read the code before
asserting a signature (Working Agreement, Rule 4).

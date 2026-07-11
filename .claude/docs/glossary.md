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
| **Proof of Immunity (PoI)** | The consensus mechanism validating cures |
| **Conjugation** | Peer-to-peer transport between nodes |
| **Epigenetic status** | Active/suppressed flag on a gene (0 active, 1 suppressed) |
| **Epigenetic Suppressor Token** | Network kill-switch that disables a harmful cure instantly |

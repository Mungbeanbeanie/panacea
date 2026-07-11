//! Scout: ultra-light background daemon. Traces process behavior (syscall anomalies,
//! memory boundary violations, I/O bursts) and accumulates a per-process trajectory score.
//! Observes only — never terminates. Crossing 100 pts suspends the target and signals the
//! local Soldier spore. See ../../.claude/docs/architecture.md, Stage 1.

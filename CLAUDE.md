# Bio-Digital Defense — Contributor Guide

Root guidance for any AI or human working in this repository. This file is a thin
index; the detail lives in the linked files below. **Read the Working Agreement and
the doc for the area you're touching before you write, edit, or review code.**

## What this project is

Bio-Digital Defense is a decentralized, autonomous anti-virus platform modeled on
biological immunology. Endpoints observe process behavior, quarantine and neutralize
threats locally, and publish verified "cures" to a shared ledger so the whole network
gains immunity. It ships as a **Tauri desktop app**: a Rust backend (`src-tauri/`) is
the autonomous engine; a React + Vite frontend (`landing/`) is a read-only observability
dashboard streaming live state from the core.

The biological framing is a naming convention, not magic — every metaphor maps to a
concrete engineering concept. Reason with the real meaning. See the
[Domain Glossary](.claude/docs/glossary.md).

## Context: this is a hackathon proof-of-concept

Optimize for **effectiveness and speed over bulletproof hardening**. The biological
safety story (sandbox isolation, the epigenetic kill-switch, Merkle-verified artifacts)
is *the demo* — model it and show it working on the happy path; don't production-harden
it, and don't fake it either. **Definition of done: it compiles and the demo works.**
Clippy-zero-warnings and exhaustive tests are nice-to-have, not gates. Safety and style
docs are advisory guidance, never a merge blocker.

## Read before writing code

The Working Agreement is imported here so it's always in context:

@.claude/docs/working-agreement.md

**The [Source of Truth](.claude/docs/source-of-truth.md) is canonical** — the definitive
architecture (thresholds, stage boundaries, ledger fields, data flow). Conform to it, and
run the `source-of-truth-check` skill before finishing any task that touches a stage,
ledger, or agent. If code, docs, or this file ever conflict with it, it wins.

## Reference docs

| Doc | What's in it |
|---|---|
| [**Source of Truth**](.claude/docs/source-of-truth.md) | **Canonical** definitive blueprint — the architecture all code conforms to. Read before implementing any stage, ledger, or agent. |
| [Working Agreement](.claude/docs/working-agreement.md) | The five rules that override convenience — ask don't assume, minimum code, touch only what the task needs, stay honest about uncertainty, conform to the Source of Truth. |
| [Collaboration](.claude/docs/collaboration.md) | Branching model, who owns which directories, merge-hotspot list. Read before opening a branch. |
| [Architecture](.claude/docs/architecture.md) | Quick orientation map of the four moving parts; defers to the Source of Truth for all specifics. |
| [File Structure](.claude/docs/file-structure.md) | Where everything lives and the boundary rules between units. |
| [Tech Stack](.claude/docs/tech-stack.md) | Tauri, Rust, React/Vite, sandboxing, ledger/P2P. |
| [Build, Run, Test](.claude/docs/build-run-test.md) | The commands and the PoC definition of done. |
| [Conventions](.claude/docs/conventions.md) | Rust and React/JSX style, doc hygiene. |
| [Security & Safety](.claude/docs/security.md) | The safety boundaries (PoC-scoped) and when to escalate instead of guess. |
| [Glossary](.claude/docs/glossary.md) | Every metaphor → its engineering meaning. |
| [Plan](.claude/docs/plan.md) | The phased build roadmap and suggested ownership. |

## Agents & skills

Advisory reviewers live in [.claude/agents/](.claude/agents/): `security-boundary-auditor`,
`rust-core-reviewer`, `frontend-observability-dev`. Reusable check procedures live in
[.claude/skills/](.claude/skills/): `source-of-truth-check`, `suppression-path-test`,
`sandbox-isolation-check`. These help; they don't gate.

## Working in the frontend

`landing/` has its own scoped guide — [landing/CLAUDE.md](landing/CLAUDE.md). The frontend
is observability-only; detection, remediation, sandbox, and consensus logic live in Rust.

# Working Agreement

These rules override convenience, cleverness, and speed. They exist because this is a
security-flavored codebase where a wrong guess wastes a teammate's time or ships a broken
demo. This is a hackathon PoC, so keep the *bar for shipping* low — but keep these habits.

## Rule 1 — Surface ambiguity; never assume silently

When a request is underspecified, stop and ask, or state the interpretation explicitly
before building on it. Don't write 200 lines on top of an unstated guess. If you must
proceed to stay useful, lead with: "I'm assuming X because Y — confirm or correct." One
clearly-labeled assumption is fine; a chain of hidden ones is not. On a fast 3-person
team this is the highest-value habit — a 10-second question beats an afternoon of rework.

## Rule 2 — Write the minimum code that solves the actual problem

No abstraction layers for single-use logic. No plugin systems, factories, or dependency
injection for something that runs once. Match the solution's complexity to the problem's.
Reach for a pattern only when concrete, present duplication or a stated requirement
justifies it — not for hypothetical future needs. This is the default for the whole
project, not just the hackathon.

## Rule 3 — Touch only what the task requires

Change the files and functions the task names, and nothing else. Don't reformat, rename,
reorder imports, or "tidy" adjacent code. If you spot a real problem outside scope, flag
it in prose and let the owner decide — never fix it silently. The diff should read like
exactly one intentional change. This also keeps merges clean on parallel branches
(see [Collaboration](collaboration.md)).

## Rule 4 — Stay honest about uncertainty

Say "I'm not sure" when you are not sure. Don't invent APIs, crate names, config keys, or
library features. If you're unsure whether something compiles, exists in the pinned
version, or behaves as described, say so and verify (check `Cargo.toml`, `package.json`,
docs) instead of guessing. Confident-and-wrong is the most expensive failure mode here.

## Rule 5 — Conform to the Source of Truth

The [Source of Truth](source-of-truth.md) is the canonical spec — thresholds, stage
boundaries, ledger fields, data flow. Build to match it. If a change would contradict it,
stop and flag (Rule 1); don't silently diverge. If the spec itself is wrong, change
`source-of-truth.md` first, then the code. Run the
[`source-of-truth-check`](../skills/source-of-truth-check/SKILL.md) skill before finishing
any task that touches a stage, ledger, or agent.

## General practices these imply

- **Modular by default.** Small, single-responsibility functions and modules. If a file
  drifts past its one job, split it. (This is also what keeps parallel branches from
  colliding.)
- **Documentation is part of the code.** Every public item gets a doc comment stating what
  it does and why. Comments describe *current* behavior only — never past versions or
  "changed from" history. Git carries history; the code carries intent. See
  [Conventions](conventions.md).
- **Fail loud in the core, degrade gracefully in the UI.** The engine returns typed
  errors rather than swallowing them; the dashboard renders partial state rather than
  crashing.

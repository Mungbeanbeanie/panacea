# Frontend Guide — landing/ (scoped)

The observability dashboard: a read-only React + Vite view that mirrors live state from the
Rust core. Root guidance is in [../CLAUDE.md](../CLAUDE.md); this file scopes the rules for
`landing/`.

## The one hard boundary

**The frontend is observability-only.** It renders state and forwards user intent via Tauri
commands. It holds **no security logic and no source of truth** — detection, remediation,
sandbox, and consensus logic all live in Rust (`src-tauri/`). If a task seems to want
logic here, that's a signal it belongs in the core — flag it (Working Agreement, Rule 1).

## Conventions

- **Function components and hooks only.** One component per file. Keep components
  presentational; push all data logic into hooks. `src/hooks/useTauriEvents.js` is the
  pattern to follow — it subscribes to Rust→UI event streams and hands components plain
  state.
- **Header comment per component:** what it visualizes and which Rust event stream or
  command feeds it.
- **Always handle "no data yet / stream dropped."** Every live view renders a sensible
  empty/partial state; never assume events have already arrived, and never crash the view.
  Fail loud in the core, **degrade gracefully in the UI.**
- **Minimum code that works** (Rule 2). The dashboard has three views — don't add a global
  state manager, router, or component library the views don't need.

## The views (all fed through `useTauriEvents.js`)

- **`EcosystemGraph.jsx`** — live process/node canvas map.
- **`LedgerTerminal.jsx`** — rolling Proof-of-Immunity event log.
- **`StrainTree.jsx`** — evolutionary phylogenetic tree of strains.

`package.json` is the source of truth for dependencies and scripts — read it, don't assume
script names or versions (Rule 4). Build/run commands are in
[../.claude/docs/build-run-test.md](../.claude/docs/build-run-test.md).

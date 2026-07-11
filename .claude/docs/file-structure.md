# File Structure

Where things live and what belongs in each unit. Directory ownership for the 3-person
team is in [Collaboration](collaboration.md).

```
bio-digital-defense/
├── CLAUDE.md                    # Root contributor guide (thin index)
├── .claude/                     # Claude dev setup: docs, agents, skills
├── .github/                     # CI/CD: compile + test binaries per platform
│
├── src-tauri/                   # Rust core (the backend engine)          [owners A + B]
│   ├── Cargo.toml               # Rust dependency manifest — source of truth for crates
│   ├── tauri.conf.json          # Tauri config: windows, capabilities/permissions
│   └── src/
│       ├── main.rs              # Entry point; registers Tauri commands + event channels
│       ├── core/mod.rs          # Shared config, constants, cross-cutting types      [A]
│       ├── agents/                                                                    [A]
│       │   ├── mod.rs           # Agent traits + shared lifecycle types
│       │   ├── scout.rs         # Scout loop: syscall/process tracing, trajectory scoring
│       │   └── soldier.rs       # Soldier spore lifecycle: wake, remediate, apoptosis
│       ├── evolution/                                                                 [B]
│       │   ├── mod.rs           # Mutation-engine orchestration + result types
│       │   ├── sandbox.rs       # MicroVM/enclave setup, teardown, isolation guarantees
│       │   └── alleles.rs       # Exploit-primitive matrix + combinatorial fuzz driver
│       └── ledger/                                                                    [B]
│           ├── mod.rs           # Ledger facade the rest of the core talks to
│           ├── client.rs        # P2P/WebSocket transport ("conjugation")
│           ├── state.rs         # Local Merkle-tree construction + proof verification
│           └── registry.rs      # Local cache of Threat + Genome tables
│
└── landing/                     # React + Vite dashboard (read-only)         [owner C]
    ├── CLAUDE.md                # Frontend-specific guidance (scoped)
    ├── package.json             # Frontend dependencies + scripts — source of truth
    ├── index.html
    └── src/
        ├── main.jsx             # Vite/React entry
        ├── App.jsx              # Application shell + layout
        ├── components/
        │   ├── EcosystemGraph.jsx   # Live process/node canvas map
        │   ├── LedgerTerminal.jsx   # Rolling Proof-of-Immunity event log
        │   └── StrainTree.jsx       # Evolutionary phylogenetic tree of strains
        └── hooks/
            └── useTauriEvents.js    # Subscribes to Rust→UI event streams
```

## Boundary rules

- **The frontend is observability-only.** It renders state and forwards user intent via
  Tauri commands. Never put detection, remediation, sandbox, or consensus logic in
  `landing/`. That logic lives in Rust. See [landing/CLAUDE.md](../../landing/CLAUDE.md).
- **Rust modules communicate through the `mod.rs` facade** of each domain (agents,
  evolution, ledger) — not by reaching into sibling files' internals. This is also what
  lets owners A and B work in parallel without editing each other's files.
- **`Cargo.toml` and `package.json` are the only sources of truth for dependencies and
  versions.** Don't assert a crate/package version from memory (Working Agreement, Rule 4)
  — read the file.

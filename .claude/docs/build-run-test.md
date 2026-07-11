# Build, Run, and Test

These are the conventional Tauri/Vite commands. Confirm them against the actual scripts in
`package.json` and the CI in `.github/` before relying on them — don't assume script names
(Working Agreement, Rule 1).

```bash
# Frontend deps (from landing/) — package manager per package.json (npm/pnpm/yarn)
npm install

# Run the full desktop app in dev (Rust core + hot-reloading UI)
npm run tauri dev

# Production build (compiles Rust + bundles UI into platform binaries)
npm run tauri build

# Rust core: check, lint, test (from src-tauri/)
cargo check
cargo clippy -- -D warnings     # warnings as failures — nice-to-have, not a gate here
cargo test

# Frontend unit tests (if configured in package.json)
npm test
```

## Definition of done (hackathon PoC)

**It compiles and the demo works.** That's the bar. If you can `cargo check` clean and the
feature does its thing when you run the app, ship it.

Clippy-zero-warnings, exhaustive tests, and full coverage are nice-to-have, not blockers.
Add a test where the logic is fiddly or safety-relevant (the suppression path is the one
place a quick check earns its keep — see the
[`suppression-path-test`](../skills/suppression-path-test/SKILL.md) skill).

If you can't run a step, say so explicitly rather than claiming it passed (Rule 4).

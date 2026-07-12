# Handoff: Panacea — Bio-Digital Defense Frontend

## Overview
Complete frontend design for Panacea, a decentralized anti-virus concept modeled on biological immunology. Six screens: public landing page, create account, log in, dashboard, protection log, and account settings — with a simulated signed-in state.

## About the Design Files
The files in this bundle are **design references created in HTML** — prototypes showing intended look and behavior, not production code to copy directly. The task is to **recreate these designs in the target codebase's environment** — a Tauri + Vite app (see `panacea/landing/`) — using its established patterns and libraries (React components in `src/`, existing hooks like `useTauriEvents`, and components like `LedgerTerminal`, `StrainTree`). Open each `.dc.html` file in a browser (keep `support.js` alongside them) to see the live design.

## Fidelity
**High-fidelity.** Colors, typography, spacing, and interactions are final. Recreate pixel-perfectly. All stats/data are simulated placeholders — wire them to real Tauri backend events.

## Screens / Views

### 1. Landing (`Panacea Landing.dc.html`)
- **Purpose**: Public marketing page; entry to auth.
- **Layout**: Full-width dark page. Nav bar (padding 22px 48px, bottom border #2c2b31) → hero (centered, animated particle-network canvas background) → live counters strip → three feature sections (Agents, Evolution Engine, Ledger, anchor-linked from nav) → CTA footer (#201f24 background).
- **Hero**: Green pulse-dot eyebrow "IMMUNE SYSTEM ONLINE" (12px, letter-spacing 2.5px, uppercase, #a9e07f) → h1 "Panacea" (Chakra Petch 96px/1 bold, letter-spacing −1px) → tagline "Your machine grows an immune system." (Chakra Petch 30px, #c5b3e6) → body paragraph (18px/1.6, #b9b6c1, max-width 620px) → button row.
- **Signed-out buttons**: "Log in" (outlined: 1px #3b3a40 border, radius 10px, padding 13px 28px) left of "Create account" (filled: #78be43 bg, #17240d text, radius 10px, padding 14px 30px).
- **Signed-in**: buttons become "Open dashboard" (filled green) + "Log out" (outlined); nav gains a filled green "Dashboard" link; footer CTA shows only "Open dashboard".
- **Hero animation**: canvas particle network — small green/lavender dots drifting with connecting lines, occasional red "threat" dot that gets surrounded and eliminated.
- **Live counters**: "Threats neutralized network-wide" (increments every ~2s) and "Active nodes"; tabular-nums.

### 2. Create Account (`Create Account.dc.html`)
- Two-column: left brand/pitch panel, right form card (#232227, border #2f2e34, radius 16px, padding 40px).
- Fields: Full name (placeholder "John Doe"), Email, Password (8-char minimum, 3-bar strength meter: weak #e08a8a / okay #d9c85a / strong #78be43).
- Validation errors inline (#e08a8a). On success: confirmation state replaces form (green-bordered card, check icon, "verification email" copy) and user is marked signed in.
- Stores: `panacea_signed_in=1`, `panacea_user_name`, `panacea_user_email` (localStorage in the prototype — replace with real auth).

### 3. Log In (`Log In.dc.html`)
- Same two-column structure as Create Account. Email + password, inline validation, redirects to Dashboard on success and sets the signed-in flag.

### 4. Dashboard (`Dashboard.dc.html`)
- **Nav** (shared by screens 4–6): logo + "Welcome, {name}." subline (links home), tabs Home / Account / Protection, circular power toggle (46px, #78be43 when on, glow `0 0 18px rgba(120,190,67,0.45)`; grey #3b3a40 when off).
- **Status banner**: full-width, "PROTECTION ACTIVE — IMMUNE SYSTEM ONLINE" (green tint) / paused state (red tint #e08a8a).
- **Stat cards** (3-col grid, gap 24px; card: #232227 bg, #2f2e34 border, radius 14px, padding 30px, centered): RAM in use (MB), CPU % + temp, GPU % + temp — values tick every 1.5s (Chakra Petch 44px bold, tabular-nums).
- **Machine stats** (4-col): Disk (with 4px progress bar, #c5b3e6 fill), Network ↓/↑ MB/s, Uptime, Battery.
- **Agent status card**: Scouts patrolling / Soldier spores dormant / Soldiers active with colored dots (green/lavender/grey).
- **Network card**: Ledger sync block number, Threat Registry "Live", Genome Registry sync time.
- **Recent activity card**: nodes + cures/min mini-panel, then timestamped event list (time in Chakra Petch 12px grey, colored dot, 14px text).
- **Running programs table**: column headers (11px uppercase grey) — Program, Scout, CPU %, Memory, Status; row dots green (nominal) / yellow #d9c85a (elevated) / lavender (self).
- **Immune ledger**: block cards (#1c1c1f inner bg) with BLCK #, age, Trajectory, Gene vector (ipfs hash, lavender), Status.

### 5. Protection (`Protection.dc.html`)
- Header row: title + "ALL SYSTEMS IMMUNE" pill (green tint).
- 4 summary cards: Threats killed, Immunities inherited, In quarantine, Allergies.
- **Soldier deployment log**: per-kill rows — threat name, date, Detected by (scout, lavender), Kill allele, Time to kill, Outcome (green).
- **Allergy list**: tolerated program with yellow dot + "Revoke allergy" outlined button (red hover).

### 6. Account (`Account.dc.html`)
- Profile card: initials avatar (64px circle, lavender tint), name, email, member-since; "Edit profile" outlined button.
- Enrolled hosts list: status dot, name, OS/detail line, Protected/Offline status; "+ Enroll a host" filled green button.
- Proof of Immunity standing: 3 stats (Blocks validated, Isolation score, Genomes contributed).
- "Log out" outlined red button → clears signed-in state, returns to landing.

## Interactions & Behavior
- **Page transitions**: cross-fade via View Transitions API (out 220ms ease-out fade, in 260ms ease-in) + per-page enter animation (320ms fade + 6px rise).
- **Auth state**: `localStorage.panacea_signed_in` ("1"/"0") drives landing-page nav/hero/footer swaps; name/email personalize greetings, avatar initials.
- **Power toggle**: pauses stat ticking, sets scouts to 0, flips banner to paused/red.
- **Hovers**: nav links brighten to #ffffff; filled buttons lighten (#78be43 → #8ecf5b); outlined buttons' borders go lavender #c5b3e6 (red #e08a8a for destructive).
- Live stats tick on intervals (1.5–2s) with clamped random walks.

## State Management
- Auth: signed-in flag, user name, user email (replace localStorage with real auth/session).
- Dashboard: on/off, ram, cpu, gpu, temps, scouts, block height, activeNodes, curesPerMin — all should come from Tauri backend events in production.
- Forms: field values, error string, submitted flag.

## Design Tokens
**Colors**
- Background: #1c1c1f · panel #232227 · inner panel/footer #201f24 · borders #2c2b31 / #2f2e34 / #3b3a40
- Text: primary #f2f1f4 · secondary #d6d3dd / #b9b6c1 / #c9c6d0 · muted #8f8c96
- Green (primary action/health): #78be43, hover #8ecf5b, light #a9e07f, dark text-on-green #17240d
- Lavender (accent/agents): #c5b3e6, hover #ddd0f2
- Warning yellow: #d9c85a · Danger red: #e08a8a
- Tints: rgba(120,190,67,0.07–0.12) green, rgba(197,179,230,0.12) lavender, rgba(224,90,90,0.07) red

**Typography**
- Display/headings/numbers: Chakra Petch (Google Fonts; 400–700)
- Body/UI: IBM Plex Sans (400–600)
- Scale: 96px hero h1 · 40px section h2 · 30px tagline · 32px page h1 · 44px stat numbers · 16px card titles · 14px body/links · 12–13px meta · 11px uppercase labels (letter-spacing 1px)
- Numbers always `font-variant-numeric: tabular-nums`

**Spacing & shape**
- Page gutter 48px; content max-width 1180px (880px on Account)
- Card radius 14px (16px auth forms, 8–10px buttons/inner cards); card padding 28–30px; grid gap 24px
- Hairline borders 1px everywhere; no shadows except the power-button glow

## Assets
No external images. Hero visual is a generated canvas particle animation. Fonts loaded from Google Fonts.

## Files
- `Panacea Landing.dc.html` — landing/marketing
- `Create Account.dc.html`, `Log In.dc.html` — auth
- `Dashboard.dc.html`, `Protection.dc.html`, `Account.dc.html` — signed-in app
- `support.js` — prototype runtime only (do not port)

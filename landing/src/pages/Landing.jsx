// Landing — public marketing page. Nav, animated hero, live counters, three feature
// sections (Agents / Evolution Engine / Ledger), CTA footer. Entry point to auth.
// From Panacea Landing.dc.html.
import { useEffect, useState } from "react";
import HeroCanvas from "../components/HeroCanvas.jsx";
import { colors, fontDisplay } from "../tokens.js";

const sectionEyebrow = { fontSize: 12, letterSpacing: 2.5, textTransform: "uppercase", color: colors.lavender, fontWeight: 600 };
const sectionHeading = { fontFamily: fontDisplay, fontSize: 38, fontWeight: 700, margin: 0, lineHeight: 1.15 };
const sectionBody = { fontSize: 16, lineHeight: 1.65, color: colors.textBody, margin: 0 };
const card = { background: colors.panel, border: `1px solid ${colors.borderPanel}`, borderRadius: 14 };

export default function Landing({ auth, navigate }) {
  const [threats, setThreats] = useState(1842907);
  const [nodes, setNodes] = useState(12408);

  useEffect(() => {
    const id = setInterval(() => {
      setThreats((t) => t + Math.floor(Math.random() * 4));
      setNodes((n) => (Math.random() < 0.25 ? n + 1 : n));
    }, 1400);
    return () => clearInterval(id);
  }, []);

  const go = (page) => (e) => {
    e.preventDefault();
    navigate(page);
  };

  return (
    <div className="pan-page" style={{ minWidth: 320, background: colors.bg, color: colors.text, overflow: "hidden" }}>
      {/* NAV */}
      <nav style={{ display: "flex", alignItems: "center", justifyContent: "space-between", gap: 24, padding: "22px 48px", position: "relative", zIndex: 3 }}>
        <div style={{ display: "flex", alignItems: "baseline", gap: 14 }}>
          <div style={{ fontFamily: fontDisplay, fontWeight: 700, fontSize: 26, letterSpacing: 0.5 }}>Panacea</div>
          <div style={{ fontSize: 12, color: colors.textFaint, letterSpacing: 0.4, textTransform: "uppercase" }}>Bio-Digital Defense</div>
        </div>
        <div style={{ display: "flex", alignItems: "center", gap: 28 }}>
          <a href="#agents" className="pan-nav-link" style={{ fontSize: 14, fontWeight: 500 }}>Agents</a>
          <a href="#engine" className="pan-nav-link" style={{ fontSize: 14, fontWeight: 500 }}>Evolution Engine</a>
          <a href="#ledger" className="pan-nav-link" style={{ fontSize: 14, fontWeight: 500 }}>Ledger</a>
          {auth.signedIn && (
            <a
              href="#dashboard"
              onClick={go("dashboard")}
              className="pan-btn-fill"
              style={{ fontSize: 14, fontWeight: 600, padding: "10px 18px", borderRadius: 8, marginLeft: 12 }}
            >
              Dashboard
            </a>
          )}
        </div>
      </nav>

      {/* HERO */}
      <header style={{ position: "relative", minHeight: 620, display: "flex", alignItems: "center", justifyContent: "center", textAlign: "center", padding: "40px 24px 80px" }}>
        <div style={{ position: "absolute", inset: 0, zIndex: 1 }}>
          <HeroCanvas />
        </div>
        <div
          style={{
            position: "absolute",
            inset: 0,
            zIndex: 2,
            background: "radial-gradient(ellipse 60% 55% at 50% 45%, rgba(28,28,31,0.55) 0%, rgba(28,28,31,0) 70%)",
            pointerEvents: "none",
          }}
        />
        <div style={{ position: "relative", zIndex: 3, maxWidth: 820, display: "flex", flexDirection: "column", alignItems: "center", gap: 26 }}>
          <div style={{ fontSize: 12, letterSpacing: 2.5, textTransform: "uppercase", color: colors.greenLight, fontWeight: 600, display: "flex", alignItems: "center", gap: 10 }}>
            <span style={{ display: "inline-block", width: 8, height: 8, borderRadius: "50%", background: colors.green }} />
            Immune system online
          </div>
          <h1 style={{ fontFamily: fontDisplay, fontSize: 96, lineHeight: 1, fontWeight: 700, margin: 0, letterSpacing: -1 }}>Panacea</h1>
          <div style={{ fontFamily: fontDisplay, fontSize: 30, lineHeight: 1.2, fontWeight: 500, color: colors.lavender }}>
            Your machine grows an immune system.
          </div>
          <p style={{ fontSize: 18, lineHeight: 1.6, color: colors.textBody, margin: 0, maxWidth: 620 }}>
            Panacea replaces static signature patching with an autonomous, decentralized swarm of agents modeled on
            biological immunology — hunting zero-days in real time across every host on the network.
          </p>
          <div style={{ display: "flex", gap: 14, marginTop: 6 }}>
            {!auth.signedIn ? (
              <>
                <a href="#login" onClick={go("login")} className="pan-btn-outline" style={{ fontSize: 16, fontWeight: 600, padding: "13px 28px", borderRadius: 10 }}>
                  Log in
                </a>
                <a href="#signup" onClick={go("signup")} className="pan-btn-fill" style={{ fontSize: 16, fontWeight: 600, padding: "14px 30px", borderRadius: 10 }}>
                  Create account
                </a>
              </>
            ) : (
              <>
                <a href="#dashboard" onClick={go("dashboard")} className="pan-btn-fill" style={{ fontSize: 16, fontWeight: 600, padding: "14px 30px", borderRadius: 10 }}>
                  Open dashboard
                </a>
                <button
                  onClick={auth.logOut}
                  className="pan-btn-outline"
                  style={{ fontSize: 16, fontWeight: 600, background: "none", cursor: "pointer", fontFamily: "inherit", padding: "13px 28px", borderRadius: 10 }}
                >
                  Log out
                </button>
              </>
            )}
          </div>
        </div>
      </header>

      {/* LIVE COUNTERS */}
      <div style={{ display: "grid", gridTemplateColumns: "repeat(3, 1fr)", gap: 1, background: colors.border, borderTop: `1px solid ${colors.border}`, borderBottom: `1px solid ${colors.border}`, position: "relative", zIndex: 3 }}>
        {[
          [threats.toLocaleString(), "Threats neutralized"],
          [nodes.toLocaleString(), "Validator nodes"],
          ["8.2 MB", "Idle RAM footprint"],
        ].map(([value, label]) => (
          <div key={label} style={{ background: colors.bg, padding: "32px 24px", textAlign: "center", display: "flex", flexDirection: "column", gap: 6 }}>
            <div style={{ fontFamily: fontDisplay, fontSize: 40, fontWeight: 700, color: colors.text, fontVariantNumeric: "tabular-nums" }}>{value}</div>
            <div style={{ fontSize: 13, color: colors.lavender, letterSpacing: 1, textTransform: "uppercase" }}>{label}</div>
          </div>
        ))}
      </div>

      {/* SECTION 1: AGENTS */}
      <section id="agents" style={{ maxWidth: 1080, margin: "0 auto", padding: "110px 32px 90px", display: "flex", flexDirection: "column", gap: 48 }}>
        <div style={{ display: "flex", flexDirection: "column", gap: 14, maxWidth: 640 }}>
          <div style={sectionEyebrow}>01 — The Agent Caste System</div>
          <h2 style={sectionHeading}>Two castes. One immune response.</h2>
          <p style={sectionBody}>Like white blood cells, Panacea's agents specialize. Sensing is cheap and constant; killing is heavy and rare.</p>
        </div>
        <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: 24 }}>
          <div className="pan-card-hover" style={{ ...card, padding: 34, display: "flex", flexDirection: "column", gap: 14, "--hover-color": colors.green }}>
            <div style={{ display: "flex", alignItems: "center", gap: 12 }}>
              <span style={{ width: 10, height: 10, borderRadius: "50%", background: colors.green, display: "inline-block" }} />
              <div style={{ fontFamily: fontDisplay, fontSize: 22, fontWeight: 600 }}>Scout Agents</div>
              <div style={{ fontSize: 11, letterSpacing: 1.5, textTransform: "uppercase", color: colors.textFaint, marginLeft: "auto" }}>Sensors</div>
            </div>
            <p style={{ fontSize: 15, lineHeight: 1.65, color: colors.textBody, margin: 0 }}>
              Ultra-lightweight daemons that never kill — they watch. Scouts trace low-level behavioral trajectories: syscall
              anomalies, memory-boundary violations, suspicious I/O bursts — and report signatures to the swarm.
            </p>
          </div>
          <div className="pan-card-hover" style={{ ...card, padding: 34, display: "flex", flexDirection: "column", gap: 14, "--hover-color": colors.lavender }}>
            <div style={{ display: "flex", alignItems: "center", gap: 12 }}>
              <span style={{ width: 10, height: 10, borderRadius: "50%", background: colors.lavender, display: "inline-block" }} />
              <div style={{ fontFamily: fontDisplay, fontSize: 22, fontWeight: 600 }}>Soldier Agents</div>
              <div style={{ fontSize: 11, letterSpacing: 1.5, textTransform: "uppercase", color: colors.textFaint, marginLeft: "auto" }}>Executioners</div>
            </div>
            <p style={{ fontSize: 15, lineHeight: 1.65, color: colors.textBody, margin: 0 }}>
              Heavy payloads that sleep as un-executed disk spores, costing you nothing. Summoned by a high-confidence alert, a
              Soldier isolates the threat, evolves a kill, executes it — then undergoes apoptosis back into a spore.
            </p>
          </div>
        </div>
      </section>

      {/* SECTION 2: EVOLUTION ENGINE */}
      <section id="engine" style={{ borderTop: `1px solid ${colors.border}`, background: colors.panelInner }}>
        <div style={{ maxWidth: 1080, margin: "0 auto", padding: "110px 32px 90px", display: "grid", gridTemplateColumns: "1fr 1fr", gap: 64, alignItems: "center" }}>
          <div style={{ display: "flex", flexDirection: "column", gap: 14 }}>
            <div style={sectionEyebrow}>02 — The Local Evolution Engine</div>
            <h2 style={sectionHeading}>
              It doesn't guess.
              <br />
              It evolves the kill.
            </h2>
            <p style={sectionBody}>
              A waking Soldier clones the target process into an isolated MicroVM sandbox, then runs an automated fuzzing
              matrix of pre-compiled structural alleles until it finds the exact lock-and-key combination that forces a
              clean crash — without ever destabilizing your host OS.
            </p>
          </div>
          <div style={{ display: "flex", flexDirection: "column", gap: 12 }}>
            {["A1 — Targeted thread exit-tokens", "A2 — Physical resource starvation", "A3 — Memory-buffer flooding"].map((row) => {
              const [tag, ...rest] = row.split(" — ");
              return (
                <div
                  key={tag}
                  className="pan-card-hover"
                  style={{ display: "flex", alignItems: "center", gap: 16, background: "#262529", border: "1px solid #302f35", borderRadius: 12, padding: "18px 22px", "--hover-color": colors.lavender }}
                >
                  <div style={{ fontFamily: fontDisplay, fontSize: 14, fontWeight: 600, color: colors.lavender, minWidth: 24 }}>{tag}</div>
                  <div style={{ fontSize: 14, color: colors.textSecondary }}>{rest.join(" — ")}</div>
                </div>
              );
            })}
            <div style={{ display: "flex", alignItems: "center", gap: 16, background: "rgba(120,190,67,0.08)", border: "1px solid rgba(120,190,67,0.45)", borderRadius: 12, padding: "18px 22px" }}>
              <div style={{ fontFamily: fontDisplay, fontSize: 14, fontWeight: 700, color: colors.greenLight, minWidth: 24 }}>✓</div>
              <div style={{ fontSize: 14, color: "#d9f0c7", fontWeight: 500 }}>Lock-and-key match — clean crash confirmed</div>
            </div>
          </div>
        </div>
      </section>

      {/* SECTION 3: LEDGER */}
      <section id="ledger" style={{ maxWidth: 1080, margin: "0 auto", padding: "110px 32px 100px", display: "flex", flexDirection: "column", gap: 48 }}>
        <div style={{ display: "flex", flexDirection: "column", gap: 14, maxWidth: 680 }}>
          <div style={sectionEyebrow}>03 — The 3-Ledger Architecture</div>
          <h2 style={sectionHeading}>Crowd-sourced immunity, secured by Proof of Immunity.</h2>
          <p style={sectionBody}>Validators must prove target-isolation metrics to write blocks. One infection observed anywhere becomes immunity everywhere.</p>
        </div>
        <div style={{ display: "grid", gridTemplateColumns: "repeat(3, 1fr)", gap: 24 }}>
          {[
            ["LEDGER 1", "State Ledger", "The cryptographic core — block headers and Merkle roots that make the system's history tamper-proof."],
            ["LEDGER 2", "Threat Registry", "Real-time behavioral signatures from Scouts. Identical cross-host matches escalate confidence and deploy Soldiers globally against zero-days."],
            ["LEDGER 3", "Genome Registry", "Confirmed threat IDs mapped to cryptographic hashes of verified Wasm kill-modules, stored on IPFS."],
          ].map(([tag, title, body]) => (
            <div key={tag} className="pan-card-hover" style={{ ...card, padding: 30, display: "flex", flexDirection: "column", gap: 12, "--hover-color": colors.lavender }}>
              <div style={{ fontFamily: fontDisplay, fontSize: 13, color: colors.textFaint, letterSpacing: 1 }}>{tag}</div>
              <div style={{ fontFamily: fontDisplay, fontSize: 20, fontWeight: 600 }}>{title}</div>
              <p style={{ fontSize: 14, lineHeight: 1.65, color: colors.textBody, margin: 0 }}>{body}</p>
            </div>
          ))}
        </div>
      </section>

      {/* CTA FOOTER */}
      <footer id="signup" style={{ borderTop: `1px solid ${colors.border}`, background: colors.panelInner }}>
        <div style={{ maxWidth: 1080, margin: "0 auto", padding: "90px 32px", display: "flex", flexDirection: "column", alignItems: "center", textAlign: "center", gap: 22 }}>
          <h2 style={{ fontFamily: fontDisplay, fontSize: 40, fontWeight: 700, margin: 0, lineHeight: 1.15 }}>Join the herd immunity.</h2>
          <p style={{ fontSize: 16, lineHeight: 1.6, color: colors.textBody, margin: 0, maxWidth: 520 }}>
            Every host that runs Panacea makes every other host harder to infect.
          </p>
          <div style={{ display: "flex", gap: 14, marginTop: 8 }}>
            {!auth.signedIn ? (
              <>
                <a href="#signup" onClick={go("signup")} className="pan-btn-fill" style={{ fontSize: 16, fontWeight: 600, padding: "14px 30px", borderRadius: 10 }}>
                  Create account
                </a>
                <a href="#login" onClick={go("login")} className="pan-btn-outline" style={{ fontSize: 16, fontWeight: 600, padding: "13px 28px", borderRadius: 10 }}>
                  Log in
                </a>
              </>
            ) : (
              <a href="#dashboard" onClick={go("dashboard")} className="pan-btn-fill" style={{ fontSize: 16, fontWeight: 600, padding: "14px 30px", borderRadius: 10 }}>
                Open dashboard
              </a>
            )}
          </div>
          <div style={{ fontSize: 12, color: colors.textFaint, marginTop: 34 }}>Panacea — an autonomous, decentralized, zero-trust anti-virus ecosystem.</div>
        </div>
      </footer>
    </div>
  );
}

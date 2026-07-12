// Protection — kill log and allergy list for this host. From Protection.dc.html.
// Kill/allergy history has no backing Tauri stream yet, so data stays as the design's
// simulated placeholders.
import SignedInNav from "../components/SignedInNav.jsx";
import { colors, fontDisplay } from "../tokens.js";

const card = { background: colors.panel, border: `1px solid ${colors.borderPanel}`, borderRadius: 14 };
const cardTitle = { fontFamily: fontDisplay, fontSize: 16, fontWeight: 600, letterSpacing: 0.5 };

const KILLS = [
  { threat: "Trojan.Keylog.Variant-C", date: "Jul 10, 13:04", scout: "Scout_11", allele: "A3 — Memory-buffer flooding", ttk: "4.2s", outcome: "Killed, apoptosis complete", color: "#78be43" },
  { threat: "Zero-day (unnamed)", date: "Jul 9, 02:31", scout: "Scout_04", allele: "A1 — Thread exit-tokens", ttk: "11.8s", outcome: "Killed, genome published", color: "#78be43" },
  { threat: "Cryptominer.XMRig.Mod", date: "Jul 6, 19:22", scout: "Scout_07", allele: "A2 — Resource starvation", ttk: "2.9s", outcome: "Killed, apoptosis complete", color: "#78be43" },
  { threat: "Ransomware.Locky.2026", date: "Jul 2, 08:15", scout: "Scout_04", allele: "Inherited — network genome", ttk: "0.4s", outcome: "Killed before execution", color: "#a9e07f" },
];

export default function Protection({ auth, navigate }) {
  return (
    <div className="pan-page" style={{ minHeight: "100vh", background: colors.bg, color: colors.text, display: "flex", flexDirection: "column" }}>
      <SignedInNav active="protection" userName={auth.userName} onNavigate={navigate} />

      <main style={{ maxWidth: 1180, width: "100%", margin: "0 auto", padding: 48, boxSizing: "border-box", display: "flex", flexDirection: "column", gap: 40 }}>
        <div style={{ display: "flex", alignItems: "flex-end", justifyContent: "space-between", gap: 24 }}>
          <div style={{ display: "flex", flexDirection: "column", gap: 8 }}>
            <h1 style={{ fontFamily: fontDisplay, fontSize: 32, fontWeight: 700, margin: 0 }}>Protection</h1>
            <p style={{ fontSize: 15, color: colors.textFaint, margin: 0 }}>Every threat this host has faced — and how it was killed.</p>
          </div>
          <div style={{ display: "flex", alignItems: "center", gap: 10, background: colors.greenTint, border: `1px solid ${colors.greenTintBorder}`, borderRadius: 10, padding: "12px 20px" }}>
            <span style={{ width: 8, height: 8, borderRadius: "50%", background: colors.green, display: "inline-block" }} />
            <span style={{ fontSize: 13, fontWeight: 600, color: colors.greenLight, letterSpacing: 1, textTransform: "uppercase" }}>All systems immune</span>
          </div>
        </div>

        {/* SUMMARY */}
        <section style={{ display: "grid", gridTemplateColumns: "repeat(4, 1fr)", gap: 24 }}>
          {[
            ["Threats killed", "37", undefined, "Lifetime, this host"],
            ["Immunities inherited", "184,209", colors.greenLight, "From the network"],
            ["In quarantine", "2", undefined, "Awaiting verdict"],
            ["Allergies", "1", colors.yellow, "Suppressed responses"],
          ].map(([label, value, color, sub]) => (
            <div key={label} style={{ ...card, padding: "22px 26px", display: "flex", flexDirection: "column", gap: 4 }}>
              <div style={{ fontSize: 11, color: colors.textFaint, letterSpacing: 1, textTransform: "uppercase" }}>{label}</div>
              <div style={{ fontFamily: fontDisplay, fontSize: 28, fontWeight: 700, fontVariantNumeric: "tabular-nums", color }}>{value}</div>
              <div style={{ fontSize: 12, color: colors.textFaint }}>{sub}</div>
            </div>
          ))}
        </section>

        {/* KILL LOG */}
        <section style={{ ...card, padding: 28, display: "flex", flexDirection: "column", gap: 16 }}>
          <div style={{ display: "flex", alignItems: "center", justifyContent: "space-between" }}>
            <div style={cardTitle}>Soldier deployment log</div>
            <div style={{ fontSize: 12, color: colors.textFaint }}>Most recent first</div>
          </div>
          {KILLS.map((k) => (
            <div key={k.threat + k.date} style={{ display: "flex", flexDirection: "column", gap: 10, padding: "18px 0", borderBottom: `1px solid ${colors.border}` }}>
              <div style={{ display: "flex", alignItems: "center", gap: 14 }}>
                <span style={{ width: 8, height: 8, borderRadius: "50%", background: k.color, display: "inline-block", flexShrink: 0 }} />
                <span style={{ fontFamily: fontDisplay, fontSize: 15, fontWeight: 600 }}>{k.threat}</span>
                <span style={{ fontSize: 12, color: colors.textFaint, marginLeft: "auto", fontVariantNumeric: "tabular-nums" }}>{k.date}</span>
              </div>
              <div style={{ display: "flex", gap: 32, paddingLeft: 22, flexWrap: "wrap" }}>
                <div style={{ display: "flex", flexDirection: "column", gap: 2, minWidth: 160 }}>
                  <span style={{ fontSize: 11, color: colors.textFaint, letterSpacing: 1, textTransform: "uppercase" }}>Detected by</span>
                  <span style={{ fontSize: 13, color: colors.lavender, fontFamily: fontDisplay }}>{k.scout}</span>
                </div>
                <div style={{ display: "flex", flexDirection: "column", gap: 2, minWidth: 200 }}>
                  <span style={{ fontSize: 11, color: colors.textFaint, letterSpacing: 1, textTransform: "uppercase" }}>Kill allele</span>
                  <span style={{ fontSize: 13, color: colors.textSecondary }}>{k.allele}</span>
                </div>
                <div style={{ display: "flex", flexDirection: "column", gap: 2, minWidth: 140 }}>
                  <span style={{ fontSize: 11, color: colors.textFaint, letterSpacing: 1, textTransform: "uppercase" }}>Time to kill</span>
                  <span style={{ fontSize: 13, color: colors.textSecondary, fontVariantNumeric: "tabular-nums" }}>{k.ttk}</span>
                </div>
                <div style={{ display: "flex", flexDirection: "column", gap: 2, minWidth: 160 }}>
                  <span style={{ fontSize: 11, color: colors.textFaint, letterSpacing: 1, textTransform: "uppercase" }}>Outcome</span>
                  <span style={{ fontSize: 13, fontWeight: 600, color: k.color }}>{k.outcome}</span>
                </div>
              </div>
            </div>
          ))}
        </section>

        {/* ALLERGY LIST */}
        <section style={{ ...card, padding: 28, display: "flex", flexDirection: "column", gap: 16, marginBottom: 40 }}>
          <div style={{ display: "flex", flexDirection: "column", gap: 4 }}>
            <div style={cardTitle}>Allergy list</div>
            <div style={{ fontSize: 13, color: colors.textFaint }}>Programs you've told Panacea to tolerate despite suspicious behavior.</div>
          </div>
          <div style={{ display: "flex", alignItems: "center", gap: 14, padding: "16px 0", borderTop: `1px solid ${colors.border}` }}>
            <span style={{ width: 8, height: 8, borderRadius: "50%", background: colors.yellow, display: "inline-block", flexShrink: 0 }} />
            <div style={{ display: "flex", flexDirection: "column", gap: 2 }}>
              <span style={{ fontSize: 14, fontWeight: 500 }}>dev-hot-reload.exe</span>
              <span style={{ fontSize: 12, color: colors.textFaint }}>Trajectory: Process.Injection.Fork — suppressed by you, Jul 8</span>
            </div>
            <button className="pan-btn-outline-danger" style={{ marginLeft: "auto", fontSize: 13, fontWeight: 600, background: "none", border: "1px solid #3b3a40", borderRadius: 8, padding: "8px 16px", cursor: "pointer", fontFamily: "inherit" }}>
              Revoke allergy
            </button>
          </div>
        </section>
      </main>
    </div>
  );
}

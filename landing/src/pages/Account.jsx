// Account — profile, enrolled hosts, Proof of Immunity standing, log out.
// From Account.dc.html. Host list has no backing Tauri stream yet, so it stays as the
// design's simulated placeholder.
import SignedInNav from "../components/SignedInNav.jsx";
import { colors, fontDisplay } from "../tokens.js";

const card = { background: colors.panel, border: `1px solid ${colors.borderPanel}`, borderRadius: 14 };
const cardTitle = { fontFamily: fontDisplay, fontSize: 16, fontWeight: 600, letterSpacing: 0.5 };

const HOSTS = [
  { name: "This device", detail: "This session · 46 scouts, 142 spores", status: "Protected", color: "#78be43" },
  { name: "Home desktop", detail: "Windows 11 · Last seen 2h ago · 38 scouts, 120 spores", status: "Protected", color: "#78be43" },
  { name: "Old ThinkPad", detail: "Ubuntu 24.04 · Last seen 12d ago", status: "Offline", color: "#8f8c96" },
];

function initialsOf(name) {
  return (name || "A").trim().split(/\s+/).map((w) => w[0]).join("").slice(0, 2).toUpperCase();
}

export default function Account({ auth, navigate }) {
  function handleLogOut() {
    auth.logOut();
    navigate("landing");
  }

  return (
    <div className="pan-page" style={{ minHeight: "100vh", background: colors.bg, color: colors.text, display: "flex", flexDirection: "column" }}>
      <SignedInNav active="account" userName={auth.userName} onNavigate={navigate} />

      <main style={{ maxWidth: 880, width: "100%", margin: "0 auto", padding: 48, boxSizing: "border-box", display: "flex", flexDirection: "column", gap: 32 }}>
        <div style={{ display: "flex", flexDirection: "column", gap: 8 }}>
          <h1 style={{ fontFamily: fontDisplay, fontSize: 32, fontWeight: 700, margin: 0 }}>Account</h1>
          <p style={{ fontSize: 15, color: colors.textFaint, margin: 0 }}>Your identity, hosts, and standing in the immune network.</p>
        </div>

        {/* PROFILE */}
        <section style={{ ...card, padding: 28, display: "flex", alignItems: "center", gap: 22 }}>
          <div style={{ width: 64, height: 64, borderRadius: "50%", background: colors.lavenderTint, border: "1px solid rgba(197,179,230,0.4)", display: "flex", alignItems: "center", justifyContent: "center", fontFamily: fontDisplay, fontSize: 22, fontWeight: 700, color: colors.lavender, flexShrink: 0 }}>
            {initialsOf(auth.userName)}
          </div>
          <div style={{ display: "flex", flexDirection: "column", gap: 4 }}>
            <div style={{ fontFamily: fontDisplay, fontSize: 20, fontWeight: 600 }}>{auth.userName}</div>
            <div style={{ fontSize: 14, color: colors.textFaint }}>{auth.userEmail || "unknown@panacea.net"} · Member since Jul 2026 · Open beta</div>
          </div>
          <button className="pan-btn-outline" style={{ marginLeft: "auto", fontSize: 13, fontWeight: 600, background: "none", border: "1px solid #3b3a40", borderRadius: 8, padding: "9px 18px", cursor: "pointer", fontFamily: "inherit", flexShrink: 0 }}>
            Edit profile
          </button>
        </section>

        {/* ENROLLED HOSTS */}
        <section style={{ ...card, padding: 28, display: "flex", flexDirection: "column", gap: 16 }}>
          <div style={{ display: "flex", alignItems: "center", justifyContent: "space-between" }}>
            <div style={cardTitle}>Enrolled hosts</div>
            <button className="pan-btn-fill" style={{ fontSize: 13, fontWeight: 600, border: "none", borderRadius: 8, padding: "9px 18px", cursor: "pointer", fontFamily: "inherit" }}>
              + Enroll a host
            </button>
          </div>
          {HOSTS.map((h) => (
            <div key={h.name} style={{ display: "flex", alignItems: "center", gap: 14, padding: "15px 0", borderBottom: `1px solid ${colors.border}` }}>
              <span style={{ width: 8, height: 8, borderRadius: "50%", background: h.color, display: "inline-block", flexShrink: 0 }} />
              <div style={{ display: "flex", flexDirection: "column", gap: 2 }}>
                <span style={{ fontSize: 14, fontWeight: 500 }}>{h.name}</span>
                <span style={{ fontSize: 12, color: colors.textFaint }}>{h.detail}</span>
              </div>
              <span style={{ fontSize: 13, color: h.color, fontWeight: 600, marginLeft: "auto" }}>{h.status}</span>
            </div>
          ))}
        </section>

        {/* VALIDATOR STANDING */}
        <section style={{ ...card, padding: 28, display: "flex", flexDirection: "column", gap: 18, marginBottom: 40 }}>
          <div style={{ display: "flex", flexDirection: "column", gap: 4 }}>
            <div style={cardTitle}>Proof of Immunity standing</div>
            <div style={{ fontSize: 13, color: colors.textFaint }}>Your hosts validate blocks by proving target-isolation metrics.</div>
          </div>
          <div style={{ display: "grid", gridTemplateColumns: "repeat(3, 1fr)", gap: 20 }}>
            {[
              ["1,042", "Blocks validated", undefined],
              ["99.7%", "Isolation score", colors.greenLight],
              ["3", "Genomes contributed", colors.lavender],
            ].map(([value, label, color]) => (
              <div key={label} style={{ display: "flex", flexDirection: "column", gap: 2 }}>
                <span style={{ fontFamily: fontDisplay, fontSize: 24, fontWeight: 700, fontVariantNumeric: "tabular-nums", color }}>{value}</span>
                <span style={{ fontSize: 12, color: colors.textFaint, letterSpacing: 1, textTransform: "uppercase" }}>{label}</span>
              </div>
            ))}
          </div>
        </section>

        <div style={{ display: "flex", justifyContent: "flex-start", marginBottom: 60 }}>
          <button onClick={handleLogOut} className="pan-btn-outline-danger" style={{ fontSize: 14, fontWeight: 600, background: "none", cursor: "pointer", fontFamily: "inherit", borderRadius: 8, padding: "10px 20px" }}>
            Log out
          </button>
        </div>
      </main>
    </div>
  );
}

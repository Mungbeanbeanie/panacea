// CreateAccount — two-column signup: brand pitch left, form right. Inline validation,
// a 3-bar password strength meter, and a confirmation state on success.
// From Create Account.dc.html.
import { useState } from "react";
import { colors, fontDisplay, fontBody } from "../tokens.js";

const EMAIL_RE = /^[^\s@]+@[^\s@]+\.[^\s@]+$/;
const STRENGTH_COLORS = ["", "#e08a5a", colors.yellow, colors.green];
const STRENGTH_LABELS = ["Password strength", "Weak — try 8+ characters", "Good — add a symbol for best strength", "Strong"];

function strengthOf(password) {
  if (!password) return 0;
  let s = 0;
  if (password.length >= 8) s++;
  if (/[A-Z]/.test(password) && /[0-9]/.test(password)) s++;
  if (password.length >= 12 && /[^A-Za-z0-9]/.test(password)) s++;
  return Math.max(1, s);
}

const inputStyle = {
  background: colors.bg,
  border: `1px solid ${colors.borderInput}`,
  borderRadius: 10,
  padding: "13px 16px",
  fontSize: 15,
  color: colors.text,
  fontFamily: fontBody,
  outline: "none",
  width: "100%",
  boxSizing: "border-box",
};

export default function CreateAccount({ auth, navigate }) {
  const [name, setName] = useState("");
  const [email, setEmail] = useState("");
  const [password, setPassword] = useState("");
  const [error, setError] = useState("");
  const [submitted, setSubmitted] = useState(false);

  const strength = strengthOf(password);

  function handleSubmit(e) {
    e.preventDefault();
    if (!name.trim()) return setError("Please enter your name.");
    if (!EMAIL_RE.test(email)) return setError("Please enter a valid email address.");
    if (password.length < 8) return setError("Password must be at least 8 characters.");
    auth.logIn(name.trim(), email.trim());
    setSubmitted(true);
  }

  const firstName = name.trim().split(" ")[0] || "agent";

  return (
    <div className="pan-page" style={{ minHeight: "100vh", background: colors.bg, color: colors.text, display: "flex", flexDirection: "column" }}>
      {/* NAV */}
      <nav style={{ display: "flex", alignItems: "center", justifyContent: "space-between", gap: 24, padding: "22px 48px" }}>
        <a
          href="#landing"
          onClick={(e) => {
            e.preventDefault();
            navigate("landing");
          }}
          className="pan-nav-link"
          style={{ display: "flex", alignItems: "baseline", gap: 14, color: colors.text }}
        >
          <span style={{ fontFamily: fontDisplay, fontWeight: 700, fontSize: 26, letterSpacing: 0.5 }}>Panacea</span>
          <span style={{ fontSize: 12, color: colors.textFaint, letterSpacing: 0.4, textTransform: "uppercase" }}>Bio-Digital Defense</span>
        </a>
        <div style={{ display: "flex", alignItems: "center", gap: 10, fontSize: 14, color: colors.textFaint }}>
          Already have an account?
          <a
            href="#login"
            onClick={(e) => {
              e.preventDefault();
              navigate("login");
            }}
            className="pan-btn-outline"
            style={{ fontSize: 14, fontWeight: 600, padding: "9px 18px", borderRadius: 8 }}
          >
            Log in
          </a>
        </div>
      </nav>

      {/* BODY */}
      <div style={{ flex: 1, display: "grid", gridTemplateColumns: "1fr 1fr", alignItems: "stretch", maxWidth: 1180, width: "100%", margin: "0 auto", gap: 64, padding: "40px 48px 80px", boxSizing: "border-box" }}>
        {/* LEFT: pitch */}
        <div style={{ display: "flex", flexDirection: "column", justifyContent: "center", gap: 28 }}>
          <h1 style={{ fontFamily: fontDisplay, fontSize: 44, fontWeight: 700, margin: 0, lineHeight: 1.1 }}>
            Enroll this machine
            <br />
            in the immune network.
          </h1>
          <p style={{ fontSize: 16, lineHeight: 1.65, color: colors.textBody, margin: 0, maxWidth: 440 }}>
            One account covers all your hosts. The moment you join, your Scouts start contributing behavioral
            signatures — and you inherit immunity from every threat the network has ever killed.
          </p>
          <div style={{ display: "flex", flexDirection: "column", gap: 16 }}>
            {[
              [colors.green, "Scouts deploy in under a minute — 8.2 MB idle footprint"],
              [colors.lavender, "Soldiers stay dormant as spores until a threat is confirmed"],
              [colors.green, "Zero-trust: your data never leaves the host, only signatures do"],
            ].map(([dot, text]) => (
              <div key={text} style={{ display: "flex", alignItems: "center", gap: 14 }}>
                <span style={{ width: 8, height: 8, borderRadius: "50%", background: dot, display: "inline-block", flexShrink: 0 }} />
                <span style={{ fontSize: 14, color: colors.textSecondary }}>{text}</span>
              </div>
            ))}
          </div>
        </div>

        {/* RIGHT: form */}
        <div style={{ display: "flex", alignItems: "center" }}>
          {!submitted ? (
            <form
              onSubmit={handleSubmit}
              style={{ width: "100%", background: colors.panel, border: `1px solid ${colors.borderPanel}`, borderRadius: 16, padding: 40, display: "flex", flexDirection: "column", gap: 20, boxSizing: "border-box" }}
            >
              <div style={{ display: "flex", flexDirection: "column", gap: 6, marginBottom: 4 }}>
                <div style={{ fontFamily: fontDisplay, fontSize: 24, fontWeight: 700 }}>Create your account</div>
                <div style={{ fontSize: 14, color: colors.textFaint }}>Free while Panacea is in open beta.</div>
              </div>

              <div style={{ display: "flex", flexDirection: "column", gap: 8 }}>
                <label htmlFor="name" style={{ fontSize: 13, fontWeight: 600, color: colors.textMuted }}>Full name</label>
                <input
                  id="name"
                  type="text"
                  placeholder="John Doe"
                  value={name}
                  onChange={(e) => {
                    setName(e.target.value);
                    setError("");
                  }}
                  className="pan-input"
                  style={inputStyle}
                />
              </div>

              <div style={{ display: "flex", flexDirection: "column", gap: 8 }}>
                <label htmlFor="email" style={{ fontSize: 13, fontWeight: 600, color: colors.textMuted }}>Email</label>
                <input
                  id="email"
                  type="email"
                  placeholder="you@example.com"
                  value={email}
                  onChange={(e) => {
                    setEmail(e.target.value);
                    setError("");
                  }}
                  className="pan-input"
                  style={inputStyle}
                />
              </div>

              <div style={{ display: "flex", flexDirection: "column", gap: 8 }}>
                <label htmlFor="password" style={{ fontSize: 13, fontWeight: 600, color: colors.textMuted }}>Password</label>
                <input
                  id="password"
                  type="password"
                  placeholder="At least 8 characters"
                  value={password}
                  onChange={(e) => {
                    setPassword(e.target.value);
                    setError("");
                  }}
                  className="pan-input"
                  style={inputStyle}
                />
                <div style={{ display: "flex", gap: 6, marginTop: 2 }}>
                  {[1, 2, 3].map((bar) => (
                    <div key={bar} style={{ flex: 1, height: 4, borderRadius: 2, background: strength >= bar ? STRENGTH_COLORS[strength] : colors.borderInput }} />
                  ))}
                </div>
                <div style={{ fontSize: 12, color: colors.textFaint }}>{STRENGTH_LABELS[strength]}</div>
              </div>

              {error && (
                <div style={{ fontSize: 13, color: colors.red, background: colors.redTint, border: `1px solid ${colors.redTintBorder}`, borderRadius: 10, padding: "12px 16px" }}>
                  {error}
                </div>
              )}

              <button type="submit" className="pan-btn-fill" style={{ fontSize: 16, fontWeight: 600, padding: "15px 30px", borderRadius: 10, border: "none", cursor: "pointer", fontFamily: fontBody, marginTop: 4 }}>
                Create account
              </button>

              <div style={{ fontSize: 12, color: colors.textFaint, lineHeight: 1.6, textAlign: "center" }}>
                By creating an account you agree to the <a href="#terms">Terms</a> and <a href="#privacy">Privacy Policy</a>.
              </div>
            </form>
          ) : (
            <div style={{ width: "100%", background: colors.panel, border: "1px solid rgba(120,190,67,0.45)", borderRadius: 16, padding: "48px 40px", display: "flex", flexDirection: "column", alignItems: "center", textAlign: "center", gap: 18, boxSizing: "border-box" }}>
              <div style={{ width: 56, height: 56, borderRadius: "50%", background: "rgba(120,190,67,0.12)", border: "1px solid rgba(120,190,67,0.5)", display: "flex", alignItems: "center", justifyContent: "center", fontSize: 26, color: colors.greenLight }}>
                ✓
              </div>
              <div style={{ fontFamily: fontDisplay, fontSize: 24, fontWeight: 700 }}>Welcome, {firstName}.</div>
              <p style={{ fontSize: 15, lineHeight: 1.65, color: colors.textBody, margin: 0, maxWidth: 360 }}>
                Your account is ready. We've sent a confirmation link to <span style={{ color: colors.lavender }}>{email}</span> — verify it to enroll your first host.
              </p>
              <a
                href="#landing"
                onClick={(e) => {
                  e.preventDefault();
                  navigate("landing");
                }}
                className="pan-btn-outline"
                style={{ fontSize: 15, fontWeight: 600, padding: "12px 26px", borderRadius: 10, marginTop: 6 }}
              >
                Back to home
              </a>
            </div>
          )}
        </div>
      </div>
    </div>
  );
}

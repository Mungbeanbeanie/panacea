// LogIn — email/password sign-in, inline validation, redirects to Dashboard on success.
// From Log In.dc.html.
import { useState } from "react";
import { colors, fontDisplay, fontBody } from "../tokens.js";

const EMAIL_RE = /^[^\s@]+@[^\s@]+\.[^\s@]+$/;

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

export default function LogIn({ auth, navigate }) {
  const [email, setEmail] = useState("");
  const [password, setPassword] = useState("");
  const [remember, setRemember] = useState(true);
  const [error, setError] = useState("");

  function handleSubmit(e) {
    e.preventDefault();
    if (!EMAIL_RE.test(email)) return setError("Please enter a valid email address.");
    if (!password) return setError("Please enter your password.");
    auth.logIn(null, email.trim());
    navigate("dashboard");
  }

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
          New to Panacea?
          <a
            href="#signup"
            onClick={(e) => {
              e.preventDefault();
              navigate("signup");
            }}
            className="pan-btn-fill"
            style={{ fontSize: 14, fontWeight: 600, padding: "10px 18px", borderRadius: 8 }}
          >
            Create account
          </a>
        </div>
      </nav>

      {/* BODY */}
      <div style={{ flex: 1, display: "flex", alignItems: "center", justifyContent: "center", padding: "40px 24px 100px" }}>
        <div style={{ width: "100%", maxWidth: 440, display: "flex", flexDirection: "column", gap: 28 }}>
          <div style={{ display: "flex", flexDirection: "column", alignItems: "center", gap: 10, textAlign: "center" }}>
            <div style={{ width: 52, height: 52, borderRadius: "50%", background: "rgba(120,190,67,0.1)", border: "1px solid rgba(120,190,67,0.4)", display: "flex", alignItems: "center", justifyContent: "center" }}>
              <span style={{ width: 12, height: 12, borderRadius: "50%", background: colors.green, display: "inline-block" }} />
            </div>
            <h1 style={{ fontFamily: fontDisplay, fontSize: 32, fontWeight: 700, margin: 0 }}>Welcome back.</h1>
            <p style={{ fontSize: 15, color: colors.textFaint, margin: 0 }}>Your Scouts kept watch while you were away.</p>
          </div>

          <form onSubmit={handleSubmit} style={{ background: colors.panel, border: `1px solid ${colors.borderPanel}`, borderRadius: 16, padding: 36, display: "flex", flexDirection: "column", gap: 20, boxSizing: "border-box" }}>
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
              <div style={{ display: "flex", alignItems: "center", justifyContent: "space-between" }}>
                <label htmlFor="password" style={{ fontSize: 13, fontWeight: 600, color: colors.textMuted }}>Password</label>
                <a href="#forgot" style={{ fontSize: 13 }}>Forgot password?</a>
              </div>
              <input
                id="password"
                type="password"
                placeholder="Your password"
                value={password}
                onChange={(e) => {
                  setPassword(e.target.value);
                  setError("");
                }}
                className="pan-input"
                style={inputStyle}
              />
            </div>

            <label style={{ display: "flex", alignItems: "center", gap: 10, fontSize: 14, color: colors.textBody, cursor: "pointer" }}>
              <input
                type="checkbox"
                checked={remember}
                onChange={(e) => setRemember(e.target.checked)}
                style={{ width: 16, height: 16, accentColor: colors.green, cursor: "pointer" }}
              />
              Keep me enrolled on this device
            </label>

            {error && (
              <div style={{ fontSize: 13, color: colors.red, background: colors.redTint, border: `1px solid ${colors.redTintBorder}`, borderRadius: 10, padding: "12px 16px" }}>
                {error}
              </div>
            )}

            <button type="submit" className="pan-btn-fill" style={{ fontSize: 16, fontWeight: 600, padding: "15px 30px", borderRadius: 10, border: "none", cursor: "pointer", fontFamily: fontBody }}>
              Log in
            </button>
          </form>

          <div style={{ fontSize: 13, color: colors.textFaint, textAlign: "center" }}>
            Protected by Proof of Immunity — this session is validated on-chain.
          </div>
        </div>
      </div>
    </div>
  );
}

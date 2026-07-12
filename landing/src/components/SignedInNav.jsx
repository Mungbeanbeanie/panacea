// SignedInNav — shared top nav for Dashboard/Protection/Account: logo + greeting,
// Home/Account/Protection tabs (active tab bold white), and an optional power toggle
// (Dashboard only). Matches the nav markup repeated across all three .dc.html screens.
import { colors, fontDisplay } from "../tokens.js";

const TABS = [
  { key: "home", label: "Home", page: "dashboard" },
  { key: "account", label: "Account", page: "account" },
  { key: "protection", label: "Protection", page: "protection" },
];

export default function SignedInNav({ active, userName, onNavigate, power }) {
  return (
    <nav
      style={{
        display: "flex",
        alignItems: "center",
        justifyContent: "space-between",
        gap: 24,
        padding: "22px 48px",
        borderBottom: `1px solid ${colors.border}`,
      }}
    >
      <a
        href="#home"
        onClick={(e) => {
          e.preventDefault();
          onNavigate("landing");
        }}
        style={{ display: "flex", flexDirection: "column", gap: 2, color: colors.text }}
        className="pan-nav-link"
      >
        <span style={{ fontFamily: fontDisplay, fontWeight: 700, fontSize: 26, letterSpacing: 0.5 }}>Panacea</span>
        <span style={{ fontSize: 12, color: colors.textFaint }}>Welcome, {userName}.</span>
      </a>
      <div style={{ display: "flex", alignItems: "center", gap: 28 }}>
        {TABS.map((tab) => (
          <a
            key={tab.key}
            href={`#${tab.key}`}
            onClick={(e) => {
              e.preventDefault();
              onNavigate(tab.page);
            }}
            className={tab.key === active ? undefined : "pan-nav-link"}
            style={{
              fontSize: 14,
              fontWeight: tab.key === active ? 600 : 500,
              color: tab.key === active ? "#ffffff" : undefined,
            }}
          >
            {tab.label}
          </a>
        ))}
        {power && (
          <button
            onClick={power.onToggle}
            title="Toggle protection"
            style={{
              width: 46,
              height: 46,
              borderRadius: "50%",
              border: "none",
              cursor: "pointer",
              background: power.on ? colors.green : colors.borderInput,
              color: colors.greenDark,
              display: "flex",
              alignItems: "center",
              justifyContent: "center",
              marginLeft: 8,
              transition: "background 0.25s, box-shadow 0.25s",
              boxShadow: power.on ? "0 0 18px rgba(120,190,67,0.45)" : "none",
            }}
          >
            <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.6" strokeLinecap="round">
              <path d="M12 3v9"></path>
              <path d="M6.2 6.2a8 8 0 1 0 11.6 0"></path>
            </svg>
          </button>
        )}
      </div>
    </nav>
  );
}

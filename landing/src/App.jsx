// Application shell — six-screen SPA (landing, auth, signed-in app) driven by a small
// in-memory router. This is a Tauri desktop window, not a browsable site, so a state
// switch stands in for a URL-based router (no react-router dependency needed).
import { useState } from "react";
import { useAuth } from "./lib/auth.js";
import Landing from "./pages/Landing.jsx";
import CreateAccount from "./pages/CreateAccount.jsx";
import LogIn from "./pages/LogIn.jsx";
import Dashboard from "./pages/Dashboard.jsx";
import Protection from "./pages/Protection.jsx";
import Account from "./pages/Account.jsx";

const PAGES = {
  landing: Landing,
  signup: CreateAccount,
  login: LogIn,
  dashboard: Dashboard,
  protection: Protection,
  account: Account,
};

export default function App() {
  const [page, setPage] = useState("landing");
  const auth = useAuth();

  const Page = PAGES[page] ?? Landing;
  return <Page auth={auth} navigate={setPage} />;
}

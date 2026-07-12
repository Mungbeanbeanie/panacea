// useAuth — signed-in state for the prototype's simulated auth. Mirrors the design's
// localStorage contract (panacea_signed_in / panacea_user_name / panacea_user_email) so
// state survives a reload; real auth/session wiring replaces this later.
import { useState } from "react";

const KEY_SIGNED_IN = "panacea_signed_in";
const KEY_NAME = "panacea_user_name";
const KEY_EMAIL = "panacea_user_email";

export function useAuth() {
  const [signedIn, setSignedIn] = useState(() => localStorage.getItem(KEY_SIGNED_IN) === "1");
  const [userName, setUserName] = useState(() => localStorage.getItem(KEY_NAME) || "agent");
  const [userEmail, setUserEmail] = useState(() => localStorage.getItem(KEY_EMAIL) || "");

  function logIn(name, email) {
    localStorage.setItem(KEY_SIGNED_IN, "1");
    if (name) localStorage.setItem(KEY_NAME, name);
    localStorage.setItem(KEY_EMAIL, email);
    setSignedIn(true);
    if (name) setUserName(name);
    setUserEmail(email);
  }

  function logOut() {
    localStorage.setItem(KEY_SIGNED_IN, "0");
    setSignedIn(false);
  }

  return { signedIn, userName, userEmail, logIn, logOut };
}

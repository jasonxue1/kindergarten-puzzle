import React from "react";
import { createRoot } from "react-dom/client";
import App from "./components/App";
import { injectCatppuccinVariables } from "./theme/catppuccin";

injectCatppuccinVariables();
const el = document.getElementById("app")!;
createRoot(el).render(<App />);
if (!new URLSearchParams(location.search).get("p")) {
  const loading = document.getElementById("loading");
  loading?.parentElement?.removeChild(loading);
}

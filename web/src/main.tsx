import React from "react";
import { createRoot } from "react-dom/client";
import App from "./ui/App";

const el = document.getElementById("app")!;
createRoot(el).render(<App />);

import React, { useEffect, useMemo, useState } from "react";
import { strings, type Lang } from "./i18n";

// WASM bootstrapping: we will dynamically import the wasm-pack JS from /public
// so Vite doesn't try to process it. See useEffect below.

const App: React.FC = () => {
  const [lang, setLang] = useState<Lang>("en");
  const t = useMemo(() => strings[lang], [lang]);
  const [showChooser, setShowChooser] = useState<boolean>(() => {
    const params = new URLSearchParams(location.search);
    return !params.get("p");
  });
  const [puzzles, setPuzzles] = useState<
    Array<{ id: string; title?: string; desc?: string }>
  >([]);

  useEffect(() => {
    // Initialize the existing WASM app which expects specific element IDs present in the DOM.
    (async () => {
      const jsUrl = new URL(
        "/pkg/puzzle_wasm.js",
        window.location.origin,
      ).toString();
      // Tell Vite not to analyze this import; let the browser load it from /public
      const mod: any = await import(/* @vite-ignore */ jsUrl);
      const init = mod?.default ?? mod;
      // Pass explicit wasm URL to avoid path resolution issues
      const wasmUrl = new URL(
        "/pkg/puzzle_wasm_bg.wasm",
        window.location.origin,
      );
      await init(wasmUrl);
    })();
  }, []);

  useEffect(() => {
    // Keep the legacy Rust UI in sync: set value on #langSel and fire change
    const sel = document.getElementById("langSel") as HTMLSelectElement | null;
    if (sel) {
      if (sel.value !== lang) sel.value = lang;
      sel.dispatchEvent(new Event("change"));
    }
  }, [lang]);

  // Load chooser list when no ?p param
  useEffect(() => {
    if (!showChooser) return;
    let cancelled = false;
    fetch("./puzzles.json")
      .then((r) => (r.ok ? r.json() : []))
      .then((list) => {
        if (cancelled) return;
        if (Array.isArray(list)) setPuzzles(list);
      })
      .catch(() => void 0);
    return () => {
      cancelled = true;
    };
  }, [showChooser]);

  // Hide chooser when user selects a local JSON file (WASM handles loading)
  useEffect(() => {
    const onChange = (e: Event) => {
      const target = e.target as HTMLElement | null;
      if (target && target.id === "file") {
        setShowChooser(false);
      }
    };
    document.addEventListener("change", onChange);
    return () => document.removeEventListener("change", onChange);
  }, []);

  const hasParamP = useMemo(
    () => new URLSearchParams(location.search).get("p") != null,
    [],
  );

  return (
    <div className="container">
      <div className="card">
        <div id="bar" className="toolbar">
          <button
            id="homeBtn"
            style={{ display: hasParamP ? "inline-block" : "none" }}
            onClick={() => (window.location.href = "./")}
          >
            {t.home}
          </button>
          <button id="resetPuzzle">{t.reset}</button>
          <button id="exportPng">{t.download}</button>
          <input type="file" id="file" accept=".json" />
          <button id="tutorBtn">{t.tutor}</button>
          <label htmlFor="langSel">{t.language}</label>
          <select
            id="langSel"
            value={lang}
            onChange={(e) => setLang(e.target.value as Lang)}
          >
            <option value="en">English</option>
            <option value="zh">中文</option>
          </select>
          <span id="help">{t.help}</span>
        </div>
        <div id="status" className="status">
          &nbsp;
        </div>
        <canvas
          id="cv"
          width={1200}
          height={800}
          style={{ background: "#fff" }}
        />
        <div id="note" style={{ padding: "8px 12px", color: "#333" }} />
      </div>
      {showChooser && (
        <div
          style={{
            position: "fixed",
            inset: 0,
            display: "flex",
            alignItems: "center",
            justifyContent: "center",
            background: "rgba(0,0,0,0.35)",
            zIndex: 20,
          }}
        >
          <div className="card" style={{ width: 520, padding: "20px 24px" }}>
            <h2 style={{ margin: "0 0 12px 0" }}>Select a Puzzle</h2>
            <ul style={{ listStyle: "none", padding: 0, margin: 0 }}>
              {puzzles.map((item) => (
                <li
                  key={item.id}
                  style={{ padding: "8px 0", borderBottom: "1px solid #eee" }}
                >
                  <a
                    href={`?p=${encodeURIComponent(item.id)}`}
                    style={{ color: "#0a66c2", textDecoration: "none" }}
                  >
                    {item.title || item.id}
                  </a>
                  {item.desc && (
                    <span style={{ color: "#666", marginLeft: 6 }}>
                      {" "}
                      — {item.desc}
                    </span>
                  )}
                </li>
              ))}
              <li style={{ padding: "8px 0", borderBottom: "1px solid #eee" }}>
                <a
                  href="#"
                  onClick={(e) => {
                    e.preventDefault();
                    const input = document.getElementById(
                      "file",
                    ) as HTMLInputElement | null;
                    input?.click();
                  }}
                  style={{ color: "#0a66c2", textDecoration: "none" }}
                >
                  Load local JSON…
                </a>
              </li>
              <li style={{ padding: "8px 0", borderBottom: "1px solid #eee" }}>
                <a
                  href="puzzle/"
                  target="_blank"
                  style={{ color: "#0a66c2", textDecoration: "none" }}
                >
                  Browse puzzle directory
                </a>
              </li>
              <li style={{ padding: "8px 0" }}>
                <a
                  href="./"
                  style={{ color: "#0a66c2", textDecoration: "none" }}
                >
                  Back to site root
                </a>
              </li>
            </ul>
          </div>
        </div>
      )}
    </div>
  );
};

export default App;

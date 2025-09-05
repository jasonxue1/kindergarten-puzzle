import React, { useEffect, useMemo, useState } from "react";
import { strings, type Lang } from "./i18n";
import { ThemeToggle } from "./theme";
import { TutorialModal } from "./Tutorial";

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
  const [showTutorial, setShowTutorial] = useState(false);

  useEffect(() => {
    // Initialize the existing WASM app which expects specific element IDs present in the DOM.
    (async () => {
      const base = import.meta.env.BASE_URL || "/";
      // Expose base to WASM before it runs so relative fetches work in dev and prod
      (window as any).__BASE_URL = base.endsWith("/") ? base : base + "/";
      const wasmUrl = `${base}pkg/puzzle_wasm_bg.wasm`;

      // Load bridge module from public to avoid Vite processing of /public assets
      async function ensureBridge(): Promise<void> {
        if ((window as any).__puzzleWasmInit) return;
        await new Promise<void>((resolve, reject) => {
          const s = document.createElement("script");
          s.type = "module";
          s.src = `${base}wasm-bridge.js`;
          s.onload = () => resolve();
          s.onerror = () => reject(new Error("Failed to load wasm-bridge.js"));
          document.head.appendChild(s);
        });
      }

      await ensureBridge();
      const init = (window as any).__puzzleWasmInit as (
        u: string,
      ) => Promise<any>;
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
        <div id="bar">
          {/* First row: controls except speed */}
          <div
            className="toolbar"
            style={{ display: "flex", gap: 8, alignItems: "center" }}
          >
            <div style={{ display: "flex", gap: 6, alignItems: "center" }}>
              <button
                id="homeBtn"
                className="icon-btn"
                style={{ display: hasParamP ? "inline-flex" : "none" }}
                title={t.home}
                onClick={() => (window.location.href = "./")}
              >
                <svg viewBox="0 0 24 24" width="18" height="18" aria-hidden>
                  <path d="M12 3l9 8h-3v9h-12v-9h-3l9-8z" />
                </svg>
                <span>{t.home}</span>
              </button>
              <button id="resetPuzzle" className="icon-btn" title={t.reset}>
                <svg viewBox="0 0 24 24" width="18" height="18" aria-hidden>
                  <path d="M12 6V3L8 7l4 4V8c2.76 0 5 2.24 5 5a5 5 0 11-9.9-1h-2.02a7 7 0 1012.92 3c0-3.87-3.13-7-7-7z" />
                </svg>
                <span>{t.reset}</span>
              </button>
              <button id="exportPng" className="icon-btn" title={t.download}>
                <svg viewBox="0 0 24 24" width="18" height="18" aria-hidden>
                  <path d="M5 20h14v-2H5v2zm7-18l-5.5 5.5h3.5V15h4V7.5H17.5L12 2z" />
                </svg>
                <span>{t.download}</span>
              </button>
              <input
                type="file"
                id="file"
                accept=".json"
                style={{ display: "none" }}
              />
              <button
                className="icon-btn"
                title={lang === "zh" ? "打开本地JSON" : "Open local JSON"}
                onClick={() =>
                  (
                    document.getElementById("file") as HTMLInputElement | null
                  )?.click()
                }
              >
                <svg viewBox="0 0 24 24" width="18" height="18" aria-hidden>
                  <path d="M19 13H5v-2h14v2zm-7-9l-5 5h3v6h4V9h3l-5-5z" />
                </svg>
                <span>{lang === "zh" ? "打开本地JSON" : "Open JSON"}</span>
              </button>
              <button
                id="tutorBtn"
                className="icon-btn"
                title={t.tutor}
                onClick={(e) => {
                  e.preventDefault();
                  setShowTutorial(true);
                }}
              >
                <svg viewBox="0 0 24 24" width="18" height="18" aria-hidden>
                  <path d="M4 6h16v2H4V6zm0 4h10v2H4v-2zm0 4h16v2H4v-2z" />
                </svg>
                <span>{t.tutor}</span>
              </button>
            </div>
            <span className="spacer" aria-hidden style={{ flex: 1 }} />
            <div style={{ display: "flex", gap: 8, alignItems: "center" }}>
              <label htmlFor="langSel">{t.language}</label>
              <select
                id="langSel"
                value={lang}
                onChange={(e) => setLang(e.target.value as Lang)}
              >
                <option value="en">English</option>
                <option value="zh">中文</option>
              </select>
              <label style={{ marginLeft: 6 }}>{t.theme}</label>
              <ThemeToggle
                labels={{
                  light: strings[lang].themeLight,
                  dark: strings[lang].themeDark,
                  auto: strings[lang].themeAuto,
                }}
              />
            </div>
          </div>

          {/* Second row: speed controls */}
          <div
            className="toolbar"
            style={{ display: "flex", gap: 16, alignItems: "center" }}
          >
            <div style={{ display: "flex", gap: 10, alignItems: "center" }}>
              <label htmlFor="fastSpeedSlider">{t.speedFast}</label>
              <input
                id="fastSpeedSlider"
                type="range"
                min={1}
                max={180}
                step={1}
                defaultValue={180}
                style={{ width: 120 }}
              />
              <input
                id="fastSpeedNumber"
                type="number"
                min={1}
                max={180}
                step={1}
                defaultValue={180}
                style={{ width: 64 }}
              />
            </div>
            <div style={{ display: "flex", gap: 10, alignItems: "center" }}>
              <label htmlFor="slowSpeedSlider">{t.speedSlow}</label>
              <input
                id="slowSpeedSlider"
                type="range"
                min={1}
                max={180}
                step={1}
                defaultValue={30}
                style={{ width: 120 }}
              />
              <input
                id="slowSpeedNumber"
                type="number"
                min={1}
                max={180}
                step={1}
                defaultValue={30}
                style={{ width: 64 }}
              />
            </div>
          </div>
          <span id="help" style={{ display: "none" }} />
        </div>
        <div id="status" className="status">
          &nbsp;
        </div>
        <canvas id="cv" width={1200} height={800} />
        <div id="note" style={{ padding: "8px 12px" }} />
      </div>
      {showChooser && (
        <div
          className="scrim"
          style={{
            position: "fixed",
            inset: 0,
            display: "flex",
            alignItems: "center",
            justifyContent: "center",
            zIndex: 20,
          }}
        >
          <div className="card" style={{ width: 520, padding: "20px 24px" }}>
            <h2 style={{ margin: "0 0 12px 0" }}>Select a Puzzle</h2>
            <ul className="chooser">
              {puzzles.map((item) => (
                <li key={item.id}>
                  <a
                    href={`?p=${encodeURIComponent(item.id)}`}
                    style={{ textDecoration: "none" }}
                  >
                    {item.title || item.id}
                  </a>
                  {item.desc && (
                    <span
                      style={{ color: "inherit", opacity: 0.8, marginLeft: 6 }}
                    >
                      {" "}
                      — {item.desc}
                    </span>
                  )}
                </li>
              ))}
              <li>
                <a
                  href="#"
                  onClick={(e) => {
                    e.preventDefault();
                    const input = document.getElementById(
                      "file",
                    ) as HTMLInputElement | null;
                    input?.click();
                  }}
                  style={{ textDecoration: "none" }}
                >
                  Load local JSON…
                </a>
              </li>
              <li>
                <a
                  href="puzzle/"
                  target="_blank"
                  style={{ textDecoration: "none" }}
                >
                  Browse puzzle directory
                </a>
              </li>
              <li>
                <a href="./" style={{ textDecoration: "none" }}>
                  Back to site root
                </a>
              </li>
            </ul>
          </div>
        </div>
      )}
      {showTutorial && (
        <TutorialModal lang={lang} onClose={() => setShowTutorial(false)} />
      )}
    </div>
  );
};

export default App;

import React, {
  useEffect,
  useMemo,
  useRef,
  useState,
  useLayoutEffect,
} from "react";
import { strings, type Lang } from "./i18n";
import { ThemeToggle } from "./theme";
import { TutorialModal } from "./Tutorial";
import Home from "./Home";

// WASM bootstrapping: we will dynamically import the wasm-pack JS from /public
// so Vite doesn't try to process it. See useEffect below.

const App: React.FC = () => {
  const [lang, setLang] = useState<Lang>(() => {
    try {
      const v = localStorage.getItem("lang");
      if (v === "zh" || v === "en") return v as Lang;
    } catch {}
    return "en";
  });
  useEffect(() => {
    try {
      localStorage.setItem("lang", lang);
    } catch {}
    document.documentElement.setAttribute("lang", lang);
  }, [lang]);
  const t = useMemo(() => strings[lang], [lang]);
  const hasParamP = useMemo(
    () => new URLSearchParams(location.search).get("p") != null,
    [],
  );
  if (!hasParamP) {
    return <Home lang={lang} setLang={setLang} />;
  }

  const [ready, setReady] = useState(false);
  const [showTutorial, setShowTutorial] = useState(false);

  const syncLang = () => {
    const sel = document.getElementById("langSel") as HTMLSelectElement | null;
    if (sel) {
      if (sel.value !== lang) sel.value = lang;
      sel.dispatchEvent(new Event("change"));
    }
    try {
      localStorage.setItem("lang", lang);
    } catch {}
  };

  useEffect(() => {
    // Initialize the existing WASM app which expects specific element IDs present in the DOM.
    (async () => {
      try {
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
            s.onerror = () =>
              reject(new Error("Failed to load wasm-bridge.js"));
            document.head.appendChild(s);
          });
        }

        await ensureBridge();
        const init = (window as any).__puzzleWasmInit as (
          u: string,
        ) => Promise<any>;
        const wasm = await init(wasmUrl);
        (window as any).__puzzleWasm = wasm;
        // Mark app as ready: show UI and remove loading overlay
        document.documentElement.classList.add("app-ready");
        const loading = document.getElementById("loading");
        if (loading && loading.parentElement)
          loading.parentElement.removeChild(loading);
        setReady(true);
        syncLang();
      } catch (err) {
        const el = document.getElementById("loadingText");
        if (el) el.textContent = strings[lang].loadFailed;
        console.error(err);
      }
    })();
  }, []);

  useEffect(() => {
    // Keep the legacy Rust UI in sync: set value on #langSel and fire change
    syncLang();
  }, [lang]);

  useEffect(() => {
    if (!ready) return;
    const params = new URLSearchParams(location.search);
    if (params.get("p") === "local") {
      const txt = sessionStorage.getItem("uploadedPuzzle");
      if (txt) {
        (window as any).__puzzleWasm?.load_puzzle_from_text(txt);
        sessionStorage.removeItem("uploadedPuzzle");
      }
    }
  }, [ready]);

  const canvasRef = useRef<HTMLCanvasElement | null>(null);
  const boardWrapRef = useRef<HTMLDivElement | null>(null);

  // ResizeObserver: keep canvas backing-store size in sync with CSS size
  useLayoutEffect(() => {
    const cv = canvasRef.current;
    const wrap = boardWrapRef.current;
    if (!cv || !wrap) return;
    const dpr = Math.max(1, Math.min(3, window.devicePixelRatio || 1));
    const resize = () => {
      const rect = wrap.getBoundingClientRect();
      const cssW = Math.max(1, Math.floor(rect.width));
      const cssH = Math.max(1, Math.floor(rect.height));
      // Set CSS size via style (already 100%) and update backing size
      const targetW = Math.floor(cssW * dpr);
      const targetH = Math.floor(cssH * dpr);
      if (cv.width !== targetW || cv.height !== targetH) {
        cv.width = targetW;
        cv.height = targetH;
        // Notify WASM to redraw without causing recursive window resize events
        window.dispatchEvent(new Event("canvas-resize"));
      }
    };
    const ro = new ResizeObserver(() => resize());
    ro.observe(wrap);
    window.addEventListener("resize", resize);
    // Panel width transitions can change available size
    const panel = document.getElementById("validationPanel");
    const onTransition = (e: TransitionEvent) => {
      if (e.propertyName === "width") resize();
    };
    panel?.addEventListener("transitionend", onTransition);
    // Initial
    resize();
    return () => {
      ro.disconnect();
      window.removeEventListener("resize", resize);
      panel?.removeEventListener("transitionend", onTransition);
    };
  }, [ready]);

  return (
    <div className="page">
      <div
        className="container"
        aria-busy={!ready}
        aria-hidden={!ready}
        style={
          !ready
            ? { filter: "blur(2px)", pointerEvents: "none", userSelect: "none" }
            : undefined
        }
      >
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
                  title={t.loadLocal}
                  onClick={() =>
                    (
                      document.getElementById("file") as HTMLInputElement | null
                    )?.click()
                  }
                >
                  <svg viewBox="0 0 24 24" width="18" height="18" aria-hidden>
                    <path d="M19 13H5v-2h14v2zm-7-9l-5 5h3v6h4V9h3l-5-5z" />
                  </svg>
                  <span>{t.loadLocal}</span>
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
          {/* Main content row: canvas + side panel */}
          <div
            className="content-row"
            style={{ display: "flex", alignItems: "stretch", minHeight: 300 }}
          >
            <div
              className="board-area"
              ref={boardWrapRef}
              style={{
                flex: 1,
                minWidth: 0,
                minHeight: 0,
                display: "flex",
                flexDirection: "column",
                /* Internal whitespace handled in Rust viewport; keep minimal outer gap */
                padding: 4,
                boxSizing: "border-box",
                gap: 8,
              }}
            >
              <div style={{ flex: 1, minHeight: 0 }}>
                <canvas id="cv" ref={canvasRef} width={1200} height={800} />
              </div>
              <div id="note" style={{ padding: "8px 12px" }} />
            </div>
            <ValidationPanel lang={lang} />
          </div>
        </div>
        {showTutorial && (
          <TutorialModal lang={lang} onClose={() => setShowTutorial(false)} />
        )}
      </div>
    </div>
  );
};

export default App;

const ValidationPanel: React.FC<{ lang: Lang }> = ({ lang }) => {
  const [open, setOpen] = useState(true);
  const t = strings[lang];
  return (
    <aside
      id="validationPanel"
      className={`side-panel ${open ? "" : "collapsed"}`}
      style={{
        width: open ? 300 : 36,
      }}
    >
      <div
        className="panel-header"
        style={{ display: "flex", alignItems: "center" }}
      >
        <button
          className="icon-btn"
          aria-label={open ? t.collapse : t.expand}
          title={open ? t.collapse : t.expand}
          onClick={() => setOpen(!open)}
          style={{
            padding: 4,
            height: 28,
            width: 28,
            display: "grid",
            placeItems: "center",
          }}
        >
          <span aria-hidden>{open ? "⟨" : "⟩"}</span>
        </button>
        {open && (
          <h3 style={{ margin: "0 0 0 8px", fontSize: 16 }}>{t.validation}</h3>
        )}
      </div>
      {open && (
        <div
          id="validationContent"
          className="panel-body"
          style={{ padding: "8px 10px", fontSize: 14 }}
        >
          <div style={{ opacity: 0.7 }}>{t.success}</div>
        </div>
      )}
    </aside>
  );
};

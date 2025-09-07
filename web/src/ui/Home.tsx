import React, { useEffect, useState } from "react";
import { strings, type Lang } from "./i18n";
import { ThemeToggle } from "./theme";

interface PuzzleInfo {
  id: string;
  title?: string;
  desc?: string;
}

const Home: React.FC<{ lang: Lang; setLang: (lang: Lang) => void }> = ({
  lang,
  setLang,
}) => {
  const t = strings[lang];
  const [puzzles, setPuzzles] = useState<PuzzleInfo[]>([]);

  useEffect(() => {
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
  }, []);

  return (
    <div className="page">
      <div className="container">
        <div
          className="card"
          style={{ display: "flex", flexDirection: "column" }}
        >
          <div
            className="toolbar"
            style={{ display: "flex", justifyContent: "flex-end", gap: 8 }}
          >
            <label htmlFor="homeLangSel">{t.language}</label>
            <select
              id="homeLangSel"
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
          <div style={{ flex: 1, overflowY: "auto", padding: "24px" }}>
            <div
              style={{ maxWidth: 720, margin: "0 auto", textAlign: "center" }}
            >
              <h1 style={{ margin: "0 0 8px" }}>Kindergarten Puzzle</h1>
              <p style={{ margin: "0 0 16px" }}>{t.landingIntro}</p>
              <p style={{ margin: "0 0 20px" }}>
                <a
                  href="https://github.com/jasonxue1/kindergarten-puzzle"
                  target="_blank"
                  rel="noopener"
                >
                  GitHub
                </a>
                {" · "}
                <a href="./LICENSE" target="_blank" rel="noopener">
                  MIT License
                </a>
              </p>
              <p style={{ margin: "0 0 20px" }}>
                <input
                  type="file"
                  id="homeFile"
                  accept=".json"
                  style={{ display: "none" }}
                  onChange={async (e) => {
                    const file = e.target.files?.[0];
                    if (!file) return;
                    const text = await file.text();
                    sessionStorage.setItem("uploadedPuzzle", text);
                    location.href = "?p=local";
                  }}
                />
                <button
                  onClick={() =>
                    (
                      document.getElementById(
                        "homeFile",
                      ) as HTMLInputElement | null
                    )?.click()
                  }
                >
                  {t.loadLocal}
                </button>
              </p>
              <h2 style={{ margin: "0 0 12px" }}>{t.selectPuzzle}</h2>
              <ul className="chooser">
                {puzzles.map((item) => (
                  <li key={item.id}>
                    <a href={`?p=${encodeURIComponent(item.id)}`}>
                      {item.title || item.id}
                    </a>
                    {item.desc && <span>— {item.desc}</span>}
                  </li>
                ))}
              </ul>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
};

export default Home;

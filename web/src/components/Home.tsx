import React, { useEffect, useState } from "react";
import { strings, type Lang } from "../i18n";
import { ThemeToggle } from "../theme/ThemeToggle";
const Home: React.FC<{ lang: Lang; setLang: (lang: Lang) => void }> = ({ lang, setLang }) => {
  const t = strings[lang];
  const [puzzles, setPuzzles] = useState<string[]>([]);

  useEffect(() => {
    let cancelled = false;
    fetch("./puzzles.json")
      .then((r) => (r.ok ? r.json() : {}))
      .then((list) => {
        if (cancelled) return;
        if (list && typeof list === "object")
          setPuzzles(Object.keys(list as Record<string, string>));
      })
      .catch(() => void 0);
    return () => {
      cancelled = true;
    };
  }, []);

  return (
    <div className="page">
      <div className="container">
        <div className="card" style={{ display: "flex", flexDirection: "column" }}>
          <div className="toolbar" style={{ display: "flex", justifyContent: "flex-end", gap: 8 }}>
            <label htmlFor="homeLangSel">{t.language}</label>
            <select id="homeLangSel" value={lang} onChange={(e) => setLang(e.target.value as Lang)}>
              <option value="en">{t.langEn}</option>
              <option value="zh">{t.langZh}</option>
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
            <div style={{ maxWidth: 720, margin: "0 auto", textAlign: "center" }}>
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
              <h2 style={{ margin: "0 0 12px" }}>{t.selectPuzzle}</h2>
              <ul className="chooser">
                {puzzles.map((name) => (
                  <li key={name}>
                    <a href={`?p=${encodeURIComponent(name)}`}>{name}</a>
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

import React from "react";
import { strings, type Lang } from "./i18n";

export const TutorialModal: React.FC<{
  lang: Lang;
  onClose: () => void;
}> = ({ lang, onClose }) => {
  const t = strings[lang];
  const tips = (t.help || "")
    .split(/[;；]/)
    .map((s) => s.trim())
    .filter(Boolean);
  return (
    <div
      className="scrim"
      style={{
        position: "fixed",
        inset: 0,
        display: "flex",
        alignItems: "center",
        justifyContent: "center",
        zIndex: 30,
      }}
      onClick={onClose}
    >
      <div
        className="card"
        style={{ width: 560, maxWidth: "90vw", padding: "20px 24px" }}
        onClick={(e) => e.stopPropagation()}
      >
        <div style={{ display: "flex", alignItems: "center" }}>
          <h2 style={{ margin: "0 0 12px 0", flex: 1 }}>
            {lang === "zh" ? "教程" : "Tutorial"}
          </h2>
          <button
            type="button"
            onClick={onClose}
            className="icon-btn"
            title={lang === "zh" ? "关闭" : "Close"}
          >
            <svg viewBox="0 0 24 24" width="18" height="18" aria-hidden>
              <path d="M18.3 5.71L12 12l6.3 6.29-1.41 1.42L10.59 13.4 4.29 19.7 2.88 18.29 9.17 12 2.88 5.71 4.29 4.29l6.3 6.3 6.29-6.3z" />
            </svg>
            <span className="sr-only">Close</span>
          </button>
        </div>
        <ul className="chooser">
          {tips.map((tip, i) => (
            <li key={i} style={{ padding: "6px 0" }}>
              {tip}
            </li>
          ))}
        </ul>
      </div>
    </div>
  );
};

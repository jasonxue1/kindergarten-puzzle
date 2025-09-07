import React from "react";
import { strings, type Lang } from "../i18n";

export const TutorModal: React.FC<{ lang: Lang; onClose: () => void }> = ({
  lang,
  onClose,
}) => {
  const t = strings[lang].tutorModal;
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
        style={{
          width: 720,
          maxWidth: "92vw",
          padding: "20px 24px",
          maxHeight: "80vh",
          overflowY: "auto",
        }}
        onClick={(e) => e.stopPropagation()}
      >
        <div style={{ display: "flex", alignItems: "center" }}>
          <h2 style={{ margin: "0 0 12px 0", flex: 1 }}>{t.title}</h2>
          <button
            type="button"
            onClick={onClose}
            className="icon-btn"
            title={t.close}
          >
            <svg viewBox="0 0 24 24" width="18" height="18" aria-hidden>
              <path
                d={
                  "M18.3 5.71L12 12l6.3 6.29-1.41 1.42L10.59 13.4 4.29 19.7 2.88 18.29 9.17 12 2.88 5.71 4.29 4.29l6.3 6.3 " +
                  "6.29-6.3z"
                }
              />
            </svg>
            <span>{t.close}</span>
          </button>
        </div>
        {t.lines.map((line, i) => (
          <p key={i} style={{ margin: "8px 0" }}>
            {line}
          </p>
        ))}
      </div>
    </div>
  );
};

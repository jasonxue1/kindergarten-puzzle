import React, { useEffect, useMemo, useState } from "react";

export type ThemePref = "light" | "dark" | "auto";
const STORAGE_KEY = "theme";

export function getSystemPrefersDark(): boolean {
  if (typeof window === "undefined" || !window.matchMedia) return false;
  return window.matchMedia("(prefers-color-scheme: dark)").matches;
}

export function applyTheme(pref: ThemePref) {
  const dark = pref === "dark" || (pref === "auto" && getSystemPrefersDark());
  document.documentElement.setAttribute("data-theme", dark ? "dark" : "light");
}

export function useTheme() {
  const [pref, setPref] = useState<ThemePref>(() => {
    try {
      const v = localStorage.getItem(STORAGE_KEY) as ThemePref | null;
      return v ?? "auto";
    } catch {
      return "auto";
    }
  });

  // Apply immediately and on system changes when auto
  useEffect(() => {
    applyTheme(pref);
    const mq = window.matchMedia("(prefers-color-scheme: dark)");
    const onChange = () => applyTheme("auto");
    let added = false;
    if (pref === "auto") {
      mq.addEventListener?.("change", onChange);
      added = true;
    }
    return () => {
      if (added) mq.removeEventListener?.("change", onChange);
    };
  }, [pref]);

  const set = useMemo(
    () => (next: ThemePref) => {
      setPref(next);
      try {
        localStorage.setItem(STORAGE_KEY, next);
      } catch {}
    },
    [],
  );

  return { pref, set } as const;
}

export const ThemeToggle: React.FC<{
  labels?: { light: string; dark: string; auto: string };
}> = ({ labels = { light: "Light", dark: "Dark", auto: "Auto" } }) => {
  const { pref, set } = useTheme();
  return (
    <div className="theme-toggle" role="group" aria-label="Theme">
      <button
        type="button"
        aria-pressed={pref === "light"}
        title={labels.light}
        onClick={() => set("light")}
      >
        {/* Sun icon */}
        <svg viewBox="0 0 24 24" aria-hidden="true">
          <path d="M6.76 4.84l-1.8-1.79-1.41 1.41 1.79 1.8 1.42-1.42zm10.48 0l1.79-1.8 1.41 1.41-1.8 1.79-1.4-1.4zM12 4V1h-0v3h0zm0 19v-3h0v3h0zM4 12H1v0h3v0zm19 0h-3v0h3v0zM6.76 19.16l-1.42 1.42-1.79-1.8 1.41-1.41 1.8 1.79zm10.48 0l1.4 1.4 1.8-1.79-1.41-1.41-1.79 1.8zM12 7a5 5 0 100 10 5 5 0 000-10z" />
        </svg>
      </button>
      <button
        type="button"
        aria-pressed={pref === "auto"}
        title={labels.auto}
        onClick={() => set("auto")}
      >
        {/* Circle half (auto) */}
        <svg viewBox="0 0 24 24" aria-hidden="true">
          <path d="M12 2a10 10 0 100 20V2z" />
        </svg>
      </button>
      <button
        type="button"
        aria-pressed={pref === "dark"}
        title={labels.dark}
        onClick={() => set("dark")}
      >
        {/* Moon icon */}
        <svg viewBox="0 0 24 24" aria-hidden="true">
          <path d="M21 12.79A9 9 0 1111.21 3a7 7 0 109.79 9.79z" />
        </svg>
      </button>
    </div>
  );
};

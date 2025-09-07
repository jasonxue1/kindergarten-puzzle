import { strings, type Lang } from "./i18n";

(function () {
  // Apply theme preference
  try {
    const pref = localStorage.getItem("theme") || "auto";
    const mq =
      window.matchMedia && window.matchMedia("(prefers-color-scheme: dark)");
    const sysDark = mq ? mq.matches : false;
    const dark = pref === "dark" || (pref === "auto" && sysDark);
    document.documentElement.setAttribute(
      "data-theme",
      dark ? "dark" : "light",
    );
  } catch {
    /* no-op */
  }

  // Determine language and update head/loader text
  try {
    const params = new URLSearchParams(location.search);
    let lang: Lang | undefined = undefined;
    const q = params.get("lang");
    if (q === "zh" || q === "en") lang = q;
    if (!lang) {
      const stored = localStorage.getItem("lang");
      if (stored === "zh" || stored === "en") lang = stored as Lang;
    }
    if (!lang) {
      lang = (navigator.language || "").toLowerCase().startsWith("zh")
        ? "zh"
        : "en";
    }
    localStorage.setItem("lang", lang);
    document.documentElement.setAttribute("lang", lang);
    const t = strings[lang];
    const loading = document.getElementById("loadingText");
    if (loading) loading.textContent = t.loading;
    const titleEl = document.querySelector("title");
    if (titleEl) titleEl.textContent = `Kindergarten Puzzle – ${t.metaTitle}`;
    const metaDesc = document.querySelector('meta[name="description"]');
    if (metaDesc) metaDesc.setAttribute("content", t.metaDesc);
  } catch {
    /* no-op */
  }
})();

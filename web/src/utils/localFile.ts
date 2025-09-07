/**
 * Save a user-provided puzzle JSON into session storage and navigate to the
 * main game page. This mirrors the in-game loader but works from the landing
 * page before the WASM module is ready.
 */
export async function loadLocalPuzzle(file: File): Promise<void> {
  const text = await file.text();
  sessionStorage.setItem("uploadedPuzzle", text);
  window.location.href = "./?p=local";
}

/** Trigger a hidden file input so the user can pick a puzzle file. */
export function openFileDialog(id: string): void {
  (document.getElementById(id) as HTMLInputElement | null)?.click();
}

/**
 * Retrieve and clear the pending uploaded puzzle from session storage.
 * Returns `null` if nothing was stored.
 */
export function takeUploadedPuzzle(): string | null {
  const txt = sessionStorage.getItem("uploadedPuzzle");
  if (txt) {
    sessionStorage.removeItem("uploadedPuzzle");
  }
  return txt;
}

/** Navigation diagnostics. Silent in production unless `localStorage["rimlight.debug"] = "1"`. */
function enabled(): boolean {
  try {
    return import.meta.env.DEV || localStorage.getItem("rimlight.debug") === "1";
  } catch {
    return import.meta.env.DEV;
  }
}

export function navlog(...args: unknown[]) {
  if (enabled()) console.debug("[NAV]", ...args);
}

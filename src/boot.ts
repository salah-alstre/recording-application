import { useSettings, subscribeSettings } from "@/stores/settings";
import { subscribeStatus } from "@/stores/recording";
import { applyLangToDocument, useLang } from "@/i18n";
import "@/styles/tokens.css";
import "@/styles/base.css";
import "@/styles/components.css";
import "@/styles/app.css";
import "@/styles/overlay.css";

/** Shared bootstrap for every window: language, settings, engine status. Returns a cleanup function. */
export async function boot(opts: { transparent?: boolean } = {}) {
  if (import.meta.env.DEV && !("__TAURI_INTERNALS__" in window)) {
    const { installDevMock } = await import("@/lib/devMock");
    installDevMock();
  }
  applyLangToDocument(useLang.getState().lang);
  if (opts.transparent) document.documentElement.classList.add("transparent-window");
  document.addEventListener("contextmenu", (e) => {
    if (!(e.target as HTMLElement)?.closest?.("input, textarea")) e.preventDefault();
  });
  const subs = [subscribeSettings(), subscribeStatus()];
  await useSettings.getState().load();
  // Single, permanent listener: it must not be tied to any page's lifetime, otherwise a page that unmounts while the
  // window is hidden would leave animations paused forever.
  const syncHidden = () => { document.documentElement.dataset.hidden = String(document.hidden); };
  document.addEventListener("visibilitychange", syncHidden);
  syncHidden();
  return () => subs.forEach((p) => void p.then((u) => u()));
}

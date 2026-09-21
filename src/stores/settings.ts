import { create } from "zustand";
import { ipc, on } from "@/lib/ipc";
import type { Settings } from "@/lib/types";
import { applyLangToDocument, useLang } from "@/i18n";

interface SettingsState {
  settings: Settings | null;
  load: () => Promise<Settings>;
  /** Applies a mutation immediately in the UI and persists it (debounced) in the backend. */
  patch: (mutate: (draft: Settings) => void, opts?: { immediate?: boolean }) => void;
  replace: (s: Settings) => void;
}

let timer: ReturnType<typeof setTimeout> | undefined;
let inflight = 0;

export const useSettings = create<SettingsState>((set, get) => ({
  settings: null,
  load: async () => {
    const s = await ipc.getSettings();
    set({ settings: s });
    applyAppearance(s);
    return s;
  },
  replace: (s) => {
    // A broadcast that predates a local change would flip it back (e.g. the language) – drop it; the save response follows.
    if (timer !== undefined || inflight > 0) return;
    set({ settings: s });
    applyAppearance(s);
  },
  patch: (mutate, opts) => {
    const cur = get().settings;
    if (!cur) return;
    const draft = structuredClone(cur);
    mutate(draft);
    set({ settings: draft });
    applyAppearance(draft);
    clearTimeout(timer);
    const persist = () => {
      timer = undefined;
      const latest = get().settings;
      if (!latest) return;
      inflight++;
      void ipc.saveSettings(latest)
        .then((saved) => { inflight--; if (inflight === 0 && timer === undefined) get().replace(saved); })
        .catch(() => { inflight--; void get().load(); });
    };
    if (opts?.immediate) persist();
    else timer = setTimeout(persist, 220);
  },
}));

export function applyAppearance(s: Settings) {
  const root = document.documentElement;
  root.dataset.theme = s.appearance.theme;
  root.style.setProperty("--accent", s.appearance.accent);
  root.style.setProperty("--ui-scale", String(s.appearance.uiScale / 100));
  if (useLang.getState().lang !== s.general.language) useLang.getState().setLang(s.general.language);
  else applyLangToDocument(s.general.language);
}

/** Keeps a window's settings in sync with changes made anywhere else (other windows, hotkeys, tray). */
export function subscribeSettings() {
  return on<Settings>("settings-changed", (s) => useSettings.getState().replace(s));
}

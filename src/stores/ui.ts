import { create } from "zustand";
import type { AppError, SessionRow } from "@/lib/types";

export type Page = "home" | "library" | "profiles" | "diagnostics" | "settings";
export type SettingsSection =
  | "general" | "recording" | "replay" | "audio" | "microphone" | "camera" | "screenshots" | "overlay"
  | "hotkeys" | "storage" | "performance" | "privacy" | "language" | "appearance" | "advanced";

interface UiState {
  page: Page;
  settingsSection: SettingsSection;
  playerClipId: string | null;
  interrupted: SessionRow[];
  error: AppError | null;
  go: (p: Page, section?: SettingsSection) => void;
  play: (id: string | null) => void;
  setInterrupted: (rows: SessionRow[]) => void;
  setError: (e: AppError | null) => void;
}

export const useUi = create<UiState>((set) => ({
  page: "home",
  settingsSection: "general",
  playerClipId: null,
  interrupted: [],
  error: null,
  go: (page, section) => set((s) => ({ page, playerClipId: null, settingsSection: section ?? s.settingsSection })),
  play: (playerClipId) => set({ playerClipId }),
  setInterrupted: (interrupted) => set({ interrupted }),
  setError: (error) => set({ error }),
}));

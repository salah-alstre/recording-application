import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type * as T from "./types";

/** Every backend call goes through here so errors arrive as structured `AppError`s. */
async function call<R>(cmd: string, args?: Record<string, unknown>): Promise<R> {
  try {
    return await invoke<R>(cmd, args);
  } catch (e) {
    throw normalizeError(e);
  }
}

export function normalizeError(e: unknown): T.AppError {
  if (e && typeof e === "object" && "code" in e && "message" in e) return e as T.AppError;
  return { code: "internal", message: String(e), hint: null };
}

export const ipc = {
  getSettings: () => call<T.Settings>("get_settings"),
  saveSettings: (settings: T.Settings) => call<T.Settings>("save_settings", { settings }),
  getStatus: () => call<T.Status>("get_status"),
  getHardware: () => call<T.Hardware | null>("get_hardware"),
  reprobeHardware: () => call<T.Hardware>("reprobe_hardware"),
  listMonitors: () => call<T.MonitorInfo[]>("list_monitors"),
  listWindows: () => call<T.WindowInfo[]>("list_windows"),
  detectGame: () => call<T.GameInfo | null>("detect_game"),
  listAudioDevices: () => call<T.Devices>("list_audio_devices"),
  listCameras: () => call<string[]>("list_cameras"),
  recommendQuality: () => call<T.Quality>("recommend_quality"),
  estimateSize: (quality: T.Quality, tracks: number, seconds: number) => call<number>("estimate_size", { quality, tracks, seconds }),

  startRecording: () => call<void>("start_recording"),
  stopRecording: () => call<void>("stop_recording"),
  toggleRecording: () => call<void>("toggle_recording"),
  pauseRecording: () => call<void>("pause_recording"),
  resumeRecording: () => call<void>("resume_recording"),
  toggleReplay: () => call<void>("toggle_replay"),
  saveReplay: () => call<void>("save_replay"),
  addMarker: () => call<void>("add_marker"),
  toggleMic: () => call<void>("toggle_mic"),
  toggleCamera: () => call<void>("toggle_camera"),
  takeScreenshot: (mode?: string) => call<string>("take_screenshot", { mode }),
  regionPick: () => call<void>("region_pick"),
  regionConfirm: (displayIndex: number, rect: T.Rect) => call<void>("region_confirm", { displayIndex, rect }),
  regionCancel: () => call<void>("region_cancel"),

  overlayHide: () => call<void>("overlay_hide"),
  overlayToggle: () => call<void>("overlay_toggle"),
  openMain: (page?: string) => call<void>("open_main", { page }),
  hideToastWindow: () => call<void>("hide_toast_window"),
  thumbsBusy: () => call<boolean>("thumbs_busy"),
  dismissError: () => call<void>("dismiss_error"),
  quitApp: () => call<void>("quit_app"),

  listClips: (query: T.ClipQuery) => call<T.Clip[]>("list_clips", { query }),
  scanLibrary: () => call<number>("scan_library"),
  getClip: (id: string) => call<T.Clip | null>("get_clip", { id }),
  setFavorite: (id: string, favorite: boolean) => call<void>("set_favorite", { id, favorite }),
  setNotes: (id: string, notes: string) => call<void>("set_notes", { id, notes }),
  renameClip: (id: string, name: string) => call<T.Clip>("rename_clip", { id, name }),
  deleteClip: (id: string) => call<void>("delete_clip", { id }),
  revealClip: (id: string) => call<void>("reveal_clip", { id }),
  revealFile: (path: string) => call<void>("reveal_file", { path }),
  openPath: (path: string) => call<void>("open_path", { path }),
  listMarkers: (clipId: string) => call<T.Marker[]>("list_markers", { clipId }),
  addClipMarker: (clipId: string, tMs: number, label: string) => call<number>("add_clip_marker", { clipId, tMs, label }),
  deleteMarker: (id: number) => call<void>("delete_marker", { id }),
  exportClip: (opts: T.ExportOpts) => call<string>("export_clip", { opts }),

  listProfiles: () => call<T.Profile[]>("list_profiles"),
  saveProfile: (profile: T.Profile) => call<T.Profile>("save_profile", { profile }),
  deleteProfile: (id: string) => call<boolean>("delete_profile", { id }),
  setActiveProfile: (id: string) => call<T.Settings>("set_active_profile", { id }),
  listGameConfigs: () => call<T.GameConfig[]>("list_game_configs"),
  saveGameConfig: (config: T.GameConfig) => call<void>("save_game_config", { config }),
  deleteGameConfig: (exe: string) => call<void>("delete_game_config", { exe }),

  storageInfo: () => call<T.StorageInfo>("storage_info"),
  hotkeyStatus: () => call<T.HotkeyStatus[]>("hotkey_status"),
  checkHotkey: (action: string, accel: string) => call<T.HotkeyStatus>("check_hotkey", { action, accel }),
  statsSubscribe: (on: boolean) => call<void>("stats_subscribe", { on }),
  levelsSubscribe: (on: boolean) => call<void>("levels_subscribe", { on }),

  listInterrupted: () => call<T.SessionRow[]>("list_interrupted"),
  recoverSession: (id: string) => call<T.Clip>("recover_session", { id }),
  discardSession: (id: string) => call<void>("discard_session", { id }),
  sessionHistory: (limit: number) => call<T.SessionRow[]>("session_history", { limit }),

  appInfo: () => call<T.AppInfo>("app_info"),
  openLogsDir: () => call<void>("open_logs_dir"),
  openFolder: (path: string) => call<void>("open_folder", { path }),
  checkUpdate: () => call<T.UpdateInfo>("check_update"),
  installUpdate: (url: string) => call<void>("install_update", { url }),
};

export function on<P>(event: string, handler: (payload: P) => void): Promise<UnlistenFn> {
  return listen<P>(event, (e) => handler(e.payload));
}

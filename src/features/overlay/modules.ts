export const OVERLAY_MODULES = [
  "record", "replay", "saveReplay", "screenshot", "mic", "system", "camera", "mixer", "source", "mode", "stats", "quality", "clips", "settings",
] as const;
export type OverlayModule = (typeof OVERLAY_MODULES)[number];

/** Compact mode always shows the four essentials, regardless of the customised module list. */
export const COMPACT_MODULES: OverlayModule[] = ["record", "replay", "screenshot", "mic"];

export const STAT_METRICS = ["fps", "low1", "cpu", "gpu", "gpuTemp", "gpuMem", "ram", "encoder", "bitrate", "recFps", "duration"] as const;
export type StatMetric = (typeof STAT_METRICS)[number];

export const DEFAULT_HOTKEYS: Record<string, string> = {
  overlay: "Alt+Z", record: "Alt+F9", saveReplay: "Alt+F10", toggleReplay: "Alt+Shift+F10", screenshot: "Alt+F1",
  toggleMic: "Ctrl+Alt+M", toggleCamera: "Ctrl+Alt+C", toggleStats: "Alt+R", marker: "Alt+F8",
};
export const HOTKEY_ACTIONS = Object.keys(DEFAULT_HOTKEYS);

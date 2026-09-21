// Dev-only: lets the UI run in a plain browser (no Tauri) with fixture data, for design review.
import { mockIPC, mockWindows } from "@tauri-apps/api/mocks";
import { emit } from "@tauri-apps/api/event";
import type { Clip, Settings } from "./types";

const q = { preset: "high", height: 1440, fps: 60, bitrateKbps: 30000, codec: "av1", encoder: "auto", rateControl: "cbr", cq: 22, speed: "balanced", container: "mp4", audioBitrateKbps: 160 };
const settings = {
  general: { language: "en", launchWithWindows: false, startMode: "normal", closeToTray: true, autoStartReplay: false, checkUpdates: false, updateFeedUrl: "", hardwareAcceleration: true, onboarded: true },
  capture: { mode: "display", displayIndex: 1, windowTitle: "", windowExe: "", region: { x: 0, y: 0, w: 1280, h: 720 }, captureCursor: true, highlightCursor: false, showClicks: false, stopWhenAppCloses: false },
  activeProfileId: "p1", replay: { durationSecs: 60 },
  audio: { systemEnabled: true, systemVolume: 100, systemDevice: "", micEnabled: true, micVolume: 100, micDevice: "", micGainDb: 0, gateEnabled: false, gateThresholdDb: -50, compressor: false, limiter: true, fallbackToDefault: true, pttMode: "off", pttKey: "V", trackSystem: true, trackMic: true },
  camera: { enabled: false, device: "", width: 1280, height: 720, fps: 30, shape: "circle", sizePct: 24, corner: "bottom-right", marginPx: 32, mirror: true },
  screenshots: { format: "png", quality: 90, dir: "", includeCursor: false, delaySecs: 0, copyToClipboard: false },
  overlay: { mode: "full", scale: "medium", modules: ["record", "replay", "saveReplay", "screenshot", "mic", "system", "camera", "mixer", "source", "mode", "quality", "stats", "clips", "settings"] },
  hotkeys: { overlay: "Alt+Z", record: "Alt+F9", saveReplay: "Alt+F10", toggleReplay: "Alt+Shift+F10", screenshot: "Alt+F1", toggleMic: "Ctrl+Alt+M", toggleCamera: "Ctrl+Alt+C", toggleStats: "Alt+R", marker: "Alt+F8" },
  storage: { recordingsDir: "D:\\Recordings", maxLibraryGb: 0, autoDelete: false, lowSpaceWarnGb: 5 },
  performance: { statsEnabled: false, statsMetrics: ["fps", "cpu", "gpu"], statsPosition: "top-right", hudEnabled: true, hudPosition: "top-left", hudIncludeInCapture: false, notifications: true },
  privacy: { protectedApps: [], action: "blackout", hideNotifications: false },
  appearance: { theme: "midnight", accent: "#7aa2ff", uiScale: 100 },
  fileNameTemplate: "{game}_{date}_{time}", logLevel: "info",
} as unknown as Settings;

const now = Date.now();
const clips: Clip[] = [
  ["Counter-Strike-2_2026-09-20_18-42-31", "Counter-Strike 2", "recording", 222000, 1.2e9], ["Half-Life_Replay", "Half-Life", "replay", 60000, 3.1e8],
  ["Desktop_Screenshot", "", "screenshot", 0, 2.4e6], ["GTA-V_2026-09-19_21-10-02", "Grand Theft Auto V", "recording", 842000, 3.4e9],
].map(([title, game, kind, dur, size], i) => ({ id: `c${i}`, path: `D:\\Recordings\\${title}.mp4`, kind, title, game, createdAt: now - i * 3.6e6, durationMs: dur, width: 2560, height: 1440, fps: 60, codec: "av1", encoder: "NVENC AV1", sizeBytes: size, favorite: i === 0, notes: "", thumbPath: "", audioTracks: 3 })) as Clip[];

const hardware = {
  gpus: [{ name: "NVIDIA GeForce RTX 4070", vendor: "nvidia", vramBytes: 12.8e9 }], cpu: "Intel(R) Core(TM) i7-14700KF", cpuThreads: 28, ramBytes: 34.1e9, os: "Windows 11", ffmpegOk: true,
  encoders: [["h264_nvenc", "nvenc", "h264", "NVIDIA NVENC H.264"], ["hevc_nvenc", "nvenc", "hevc", "NVIDIA NVENC H.265"], ["av1_nvenc", "nvenc", "av1", "NVIDIA NVENC AV1"], ["libx264", "cpu", "h264", "CPU x264 H.264"]].map(([id, vendor, codec, label]) => ({ id, vendor, codec, label, available: true })),
};
const status = { state: "idle", recording: null, replay: null, error: null, game: null, profileId: "p1", target: { mode: "display", label: "", displayIndex: 1, width: 0, height: 0 } };

export function installDevMock() {
  (window as unknown as { __emit: typeof emit }).__emit = emit; // lets the console trigger backend events
  mockWindows("main");
  let st: Record<string, unknown> = { ...status };
  mockIPC((cmd, args) => {
    const a = (args ?? {}) as Record<string, unknown>;
    switch (cmd) {
      case "get_settings": return settings;
      case "save_settings": return a.settings;
      case "get_status": return st;
      case "get_hardware": return hardware;
      case "list_monitors": return [{ index: 1, name: "Display 1", width: 2560, height: 1440, refreshHz: 165, x: 0, y: 0, primary: true }, { index: 2, name: "Display 2", width: 1920, height: 1080, refreshHz: 60, x: 2560, y: 0, primary: false }];
      case "list_profiles": return [{ id: "p1", name: "Competitive Games", quality: q, micEnabled: true, systemEnabled: true, cameraEnabled: false, autoApps: ["cs2.exe"], builtin: true }, { id: "p2", name: "Desktop Tutorial", quality: { ...q, codec: "h264", fps: 30 }, micEnabled: true, systemEnabled: true, cameraEnabled: true, autoApps: [], builtin: true }];
      case "list_clips": return clips;
      case "storage_info": return { dir: "D:\\Recordings", exists: true, freeBytes: 412e9, totalBytes: 1e12, libraryBytes: 5.2e9, clipCount: 4 };
      case "list_windows": return [{ hwnd: 1, title: "Counter-Strike 2", exe: "cs2.exe", exePath: "", pid: 1, x: 0, y: 0, width: 2560, height: 1440, fullscreen: true }];
      case "list_audio_devices": return { inputs: [{ name: "Microphone (Realtek)", isDefault: true }], outputs: [{ name: "Speakers (Realtek)", isDefault: true }] };
      case "list_cameras": return ["Iriun Webcam"];
      case "list_interrupted": case "list_game_configs": case "session_history": case "hotkey_status": return [];
      case "app_info": return { version: "0.1.0", dataDir: "C:\\Users\\me\\AppData\\Roaming\\Rimlight", logsDir: "", ffmpegOk: true };
      case "start_recording": case "toggle_recording": st = { ...status, state: "recording", recording: { path: "", startedMs: Date.now(), elapsedBaseMs: 0, runningSinceMs: Date.now(), encoder: "NVIDIA NVENC AV1", width: 2560, height: 1440, fps: 60, game: "" } }; return null;
      case "stop_recording": st = { ...status }; return null;
      case "toggle_replay": st = { ...st, replay: st.replay ? null : { sinceMs: Date.now(), durationSecs: 60, encoder: "NVENC", saving: false } }; return null;
      case "plugin:event|listen": return 0;
      default: return null;
    }
  }, { shouldMockEvents: true });
}

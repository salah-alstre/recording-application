// Mirrors of the Rust DTOs (camelCase over IPC).
export type Lang = "en" | "ar";
export type RecState = "idle" | "preparing" | "recording" | "paused" | "stopping" | "error";

export interface Rect { x: number; y: number; w: number; h: number }

export interface Quality {
  preset: "low" | "medium" | "high" | "ultra" | "custom";
  height: number;
  fps: number;
  bitrateKbps: number;
  codec: "h264" | "hevc" | "av1";
  encoder: "auto" | "nvenc" | "amf" | "qsv" | "cpu";
  rateControl: "cbr" | "vbr" | "cqp";
  cq: number;
  speed: "fast" | "balanced" | "quality";
  container: "mp4" | "mkv";
  audioBitrateKbps: number;
}

export interface Settings {
  general: {
    language: Lang; launchWithWindows: boolean; startMode: "normal" | "minimized" | "tray";
    closeToTray: boolean; autoStartReplay: boolean; checkUpdates: boolean; updateFeedUrl: string;
    hardwareAcceleration: boolean; onboarded: boolean;
  };
  capture: {
    mode: "game" | "window" | "display" | "region" | "active";
    displayIndex: number; windowTitle: string; windowExe: string; region: Rect;
    captureCursor: boolean; highlightCursor: boolean; showClicks: boolean; stopWhenAppCloses: boolean;
  };
  activeProfileId: string;
  replay: { durationSecs: number };
  audio: {
    systemEnabled: boolean; systemVolume: number; systemDevice: string;
    micEnabled: boolean; micVolume: number; micDevice: string; micGainDb: number;
    gateEnabled: boolean; gateThresholdDb: number; compressor: boolean; limiter: boolean;
    fallbackToDefault: boolean; pttMode: "off" | "push_to_talk" | "push_to_mute"; pttKey: string;
    trackSystem: boolean; trackMic: boolean;
  };
  camera: {
    enabled: boolean; device: string; width: number; height: number; fps: number;
    shape: "circle" | "rounded" | "square"; sizePct: number;
    corner: "top-left" | "top-right" | "bottom-left" | "bottom-right"; marginPx: number; mirror: boolean;
  };
  screenshots: { format: "png" | "jpeg" | "webp"; quality: number; dir: string; includeCursor: boolean; delaySecs: number; copyToClipboard: boolean };
  overlay: { mode: "compact" | "full"; scale: "small" | "medium" | "large"; modules: string[] };
  hotkeys: Record<string, string>;
  storage: { recordingsDir: string; maxLibraryGb: number; autoDelete: boolean; lowSpaceWarnGb: number };
  performance: {
    statsEnabled: boolean; statsMetrics: string[]; statsPosition: Corner; hudEnabled: boolean; hudPosition: Corner;
    hudIncludeInCapture: boolean; notifications: boolean;
  };
  privacy: { protectedApps: string[]; action: "pause" | "blackout"; hideNotifications: boolean };
  appearance: { theme: "midnight" | "graphite" | "oled" | "glass"; accent: string; uiScale: number };
  fileNameTemplate: string;
  logLevel: string;
}
export type Corner = "top-left" | "top-right" | "bottom-left" | "bottom-right";

export interface AppError { code: string; message: string; hint?: string | null }

export interface GameInfo { name: string; exe: string; pid: number; hwnd: number; width: number; height: number; fullscreen: boolean; displayIndex: number }
export interface RecInfo { path: string; startedMs: number; elapsedBaseMs: number; runningSinceMs: number | null; encoder: string; width: number; height: number; fps: number; game: string }
export interface ReplayInfo { sinceMs: number; durationSecs: number; encoder: string; saving: boolean }
export interface Status {
  state: RecState;
  recording: RecInfo | null;
  replay: ReplayInfo | null;
  target: { mode: string; label: string; displayIndex: number; width: number; height: number };
  game: GameInfo | null;
  error: AppError | null;
  profileId: string;
}

export interface MonitorInfo { index: number; name: string; width: number; height: number; refreshHz: number; x: number; y: number; primary: boolean }
export interface WindowInfo { hwnd: number; title: string; exe: string; exePath: string; pid: number; x: number; y: number; width: number; height: number; fullscreen: boolean }
export interface EncoderInfo { id: string; vendor: "nvenc" | "amf" | "qsv" | "cpu"; codec: string; label: string; available: boolean; error?: string | null }
export interface GpuInfo { name: string; vendor: string; vramBytes: number }
export interface Hardware { gpus: GpuInfo[]; cpu: string; cpuThreads: number; ramBytes: number; os: string; encoders: EncoderInfo[]; ffmpegOk: boolean }
export interface DeviceEntry { name: string; isDefault: boolean }
export interface Devices { inputs: DeviceEntry[]; outputs: DeviceEntry[] }

export interface Profile {
  id: string; name: string; quality: Quality; micEnabled: boolean; systemEnabled: boolean; cameraEnabled: boolean; autoApps: string[]; builtin: boolean;
}
export interface GameConfig { exe: string; name: string; profileId: string; camera: Settings["camera"] | null }

export interface Clip {
  id: string; path: string; kind: "recording" | "replay" | "screenshot"; title: string; game: string; createdAt: number; durationMs: number;
  width: number; height: number; fps: number; codec: string; encoder: string; sizeBytes: number; favorite: boolean; notes: string; thumbPath: string; audioTracks: number;
}
export interface ClipQuery { section: string; search: string; sort: string }
export interface Marker { id: number; clipId: string; tMs: number; label: string }
export interface SessionRow { id: string; tmpPath: string; finalPath: string; kind: string; startedAt: number; game: string }

export interface StorageInfo { dir: string; exists: boolean; freeBytes: number; totalBytes: number; libraryBytes: number; clipCount: number }
export interface HotkeyStatus { action: string; accel: string; ok: boolean; problem: "invalid" | "duplicate" | "in_use" | "empty" | null; conflictsWith: string | null }
export interface AppInfo { version: string; dataDir: string; logsDir: string; ffmpegOk: boolean }
export interface UpdateInfo { current: string; latest: string; available: boolean; notes: string; url: string }

export interface SystemStats {
  cpuPct: number; ramUsed: number; ramTotal: number; gpuPct: number | null; gpuTempC: number | null; gpuMemUsed: number | null; gpuMemTotal: number | null;
  encoderPct: number | null; appMem: number; appCpuPct: number;
}
export interface SinkStats { bitrateKbps: number; encoderFps: number; speed: number; droppedFrames: number; droppedPct: number; sizeBytes: number; writeKbps: number; encoder: string }
export interface StatsPayload {
  system: SystemStats;
  capture: { fps: number; low1: number; width: number; height: number; regulatorSkipped: number } | null;
  recording: SinkStats | null; replay: SinkStats | null; audioLatencyMs: number; audioLateTicks: number;
}

export interface ToastPayload { id: string; kind: "info" | "success" | "warning" | "error"; code: string; params: Record<string, unknown>; path?: string | null }

export interface ExportOpts {
  id: string; startMs: number; endMs: number; crop: Rect | null; rotate: number; mute: boolean; volumePct: number;
  text: { text: string; x: number; y: number; size: number } | null;
}

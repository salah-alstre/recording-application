import { useCallback, useEffect, useRef, useState } from "react";
import { AnimatePresence, motion } from "motion/react";
import { AlertTriangle, Camera, Check, FolderOpen, Info, Mic, MicOff, RotateCcw } from "lucide-react";
import { useT, describeError, useLang, has, type Key } from "@/i18n";
import { useSettings } from "@/stores/settings";
import { useRecording, elapsedMs } from "@/stores/recording";
import { assetUrl, useNow, useStats, useTauriEvent } from "@/hooks/hooks";
import { formatBytes, formatClock } from "@/lib/format";
import { ipc } from "@/lib/ipc";
import type { StatsPayload, ToastPayload } from "@/lib/types";

/** ● REC 00:12:34 – small pill with the state of everything that is running. */
export function HudPill() {
  const t = useT();
  const status = useRecording((s) => s.status);
  const s = useSettings((x) => x.settings);
  const recording = status.state === "recording" || status.state === "paused";
  const now = useNow(status.state === "recording", 250);
  if (!s) return null;
  return (
    <div className="hud-pill" data-state={status.state}>
      {recording ? (
        <>
          <span className={`dot ${status.state === "recording" ? "rec" : ""}`} />
          <span className="hud-label">{status.state === "paused" ? t("rec.paused") : "REC"}</span>
          <span className="mono num hud-timer">{formatClock(elapsedMs(status, now))}</span>
        </>
      ) : (
        <><RotateCcw size={14} /><span className="hud-label">{t("replay.on")}</span></>
      )}
      <span className="hud-icons">
        {s.audio.micEnabled ? <Mic size={14} /> : <MicOff size={14} className="faint" />}
        {s.camera.enabled && <Camera size={14} />}
        {status.replay && recording && <RotateCcw size={14} />}
      </span>
    </div>
  );
}

const STAT_LABEL: Record<string, Key> = {
  fps: "perf.m.fps", low1: "perf.m.low1", cpu: "perf.m.cpu", gpu: "perf.m.gpu", gpuTemp: "perf.m.gpuTemp", gpuMem: "perf.m.gpuMem",
  ram: "perf.m.ram", encoder: "perf.m.encoder", bitrate: "perf.m.bitrate", recFps: "perf.m.recFps", duration: "perf.m.duration",
};

export function statValue(metric: string, s: StatsPayload, status: ReturnType<typeof useRecording.getState>["status"], now: number): string | null {
  const sink = s.recording ?? s.replay;
  switch (metric) {
    case "fps": return s.capture ? `${Math.round(s.capture.fps)}` : null;
    case "low1": return s.capture ? `${Math.round(s.capture.low1)}` : null;
    case "cpu": return `${Math.round(s.system.cpuPct)}%`;
    case "gpu": return s.system.gpuPct != null ? `${s.system.gpuPct}%` : null;
    case "gpuTemp": return s.system.gpuTempC != null ? `${s.system.gpuTempC}°C` : null;
    case "gpuMem": return s.system.gpuMemUsed != null ? formatBytes(s.system.gpuMemUsed, 1) : null;
    case "ram": return `${formatBytes(s.system.ramUsed, 0)}`;
    case "encoder": return s.system.encoderPct != null ? `${s.system.encoderPct}%` : null;
    case "bitrate": return sink ? `${(sink.bitrateKbps / 1000).toFixed(1)} Mbps` : null;
    case "recFps": return sink ? `${Math.round(sink.encoderFps)}` : null;
    case "duration": return status.recording ? formatClock(elapsedMs(status, now)) : null;
    default: return null;
  }
}

export function StatsHud() {
  const t = useT();
  const s = useSettings((x) => x.settings);
  const status = useRecording((x) => x.status);
  const stats = useStats(true);
  const now = useNow(true, 1000);
  if (!s) return null;
  const right = s.performance.statsPosition.endsWith("right");
  return (
    <div className={`stats-hud${right ? " right" : ""}`} aria-live="off">
      {s.performance.statsMetrics.map((m) => {
        const v = stats ? statValue(m, stats, status, now) : null;
        return <div key={m} className="stats-row"><span className="stats-k">{t(STAT_LABEL[m] ?? "perf.m.fps")}</span><span className="stats-v mono num">{v ?? t("common.na")}</span></div>;
      })}
      {!stats && <div className="stats-row faint">{t("common.loading")}</div>}
    </div>
  );
}

interface Live extends ToastPayload { ttl: number }

export function ToastStack() {
  const t = useT();
  const lang = useLang((s) => s.lang);
  const [items, setItems] = useState<Live[]>([]);
  const timers = useRef(new Map<string, number>());

  const dismiss = useCallback((id: string) => {
    window.clearTimeout(timers.current.get(id));
    timers.current.delete(id);
    setItems((cur) => cur.filter((x) => x.id !== id));
  }, []);

  useTauriEvent<ToastPayload>("toast", (p) => {
    const ttl = p.kind === "error" ? 9000 : p.kind === "warning" ? 6500 : 4200;
    setItems((cur) => [...cur.filter((x) => x.code !== p.code || x.kind !== p.kind), { ...p, ttl }].slice(-3));
    timers.current.set(p.id, window.setTimeout(() => dismiss(p.id), ttl));
  });

  useEffect(() => { if (items.length === 0) void ipc.hideToastWindow().catch(() => undefined); }, [items.length]);

  const text = (p: ToastPayload): { title: string; body: string } => {
    const params = p.params as Record<string, string | number>;
    const key = `toast.${p.code}`;
    if (p.kind === "error") {
      const d = describeError(lang, { code: p.code, message: String(params.message ?? p.code), hint: (params.hint as string) ?? "" });
      return { title: has(lang, `toast.${p.code}`) ? t(key as Key, params) : d.title, body: d.hint };
    }
    return { title: has(lang, key) ? t(key as Key, params) : String(params.message ?? p.code), body: has(lang, `${key}.body`) ? t(`${key}.body` as Key, params) : "" };
  };

  return (
    <div className="toast-stack">
      <AnimatePresence initial={false}>
        {items.map((p) => {
          const { title, body } = text(p);
          const isImg = p.code === "screenshot_saved" && p.path;
          return (
            <motion.button key={p.id} layout className={`toast ${p.kind}`} initial={{ opacity: 0, y: 24, scale: 0.94 }} animate={{ opacity: 1, y: 0, scale: 1 }} exit={{ opacity: 0, x: lang === "ar" ? -30 : 30, scale: 0.96 }}
              transition={{ type: "spring", stiffness: 420, damping: 32 }} onClick={() => { if (p.path) void ipc.openPath(p.path).catch(() => void ipc.revealFile(p.path!)); dismiss(p.id); }} aria-live="polite">
              {isImg ? <img className="toast-thumb" src={assetUrl(p.path!)} alt="" /> : (
                <span className="toast-ico">{p.kind === "error" || p.kind === "warning" ? <AlertTriangle size={18} /> : p.kind === "success" ? <Check size={18} /> : <Info size={18} />}</span>)}
              <span className="toast-text"><span className="toast-title">{title}</span>{body && <span className="toast-body">{body}</span>}{p.path && !isImg && <span className="toast-body"><FolderOpen size={11} style={{ verticalAlign: -1 }} /> {t("toast.clickToOpen")}</span>}</span>
              <span className="toast-bar" style={{ animationDuration: `${p.ttl}ms` }} />
            </motion.button>
          );
        })}
      </AnimatePresence>
    </div>
  );
}

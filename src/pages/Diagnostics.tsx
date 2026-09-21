import { useEffect, useRef, useState } from "react";
import { Activity } from "lucide-react";
import { useT, useLang } from "@/i18n";
import { useStats, useVisible } from "@/hooks/hooks";
import { useRecording } from "@/stores/recording";
import { formatBytes, formatDate } from "@/lib/format";
import { ipc } from "@/lib/ipc";
import type { SessionRow, StatsPayload } from "@/lib/types";
import { InfoBanner } from "@/components/overlays";

function Spark({ data, max }: { data: number[]; max?: number }) {
  const m = max ?? Math.max(1, ...data);
  const pts = data.map((v, i) => `${(i / Math.max(1, data.length - 1)) * 100},${100 - (Math.min(v, m) / m) * 100}`).join(" ");
  return (
    <svg className="spark" viewBox="0 0 100 100" preserveAspectRatio="none" aria-hidden>
      <polyline points={pts} fill="none" stroke="var(--accent)" strokeWidth="2.5" vectorEffect="non-scaling-stroke" strokeLinejoin="round" />
    </svg>
  );
}

function Tile({ label, value, unit, history, hint }: { label: string; value: string; unit?: string; history?: number[]; hint?: string }) {
  return (
    <div className="diag-tile" title={hint}>
      <div className="diag-label">{label}</div>
      <div className="diag-value mono num">{value}<span className="diag-unit">{unit}</span></div>
      {history && <Spark data={history} />}
    </div>
  );
}

export function Diagnostics() {
  const t = useT();
  const lang = useLang((s) => s.lang);
  const visible = useVisible();
  const stats = useStats(visible);
  const status = useRecording((s) => s.status);
  const hist = useRef<Record<string, number[]>>({});
  const [, force] = useState(0);
  const [sessions, setSessions] = useState<SessionRow[]>([]);
  useEffect(() => { void ipc.sessionHistory(8).then(setSessions).catch(() => undefined); }, [status.state]);

  useEffect(() => {
    if (!stats) return;
    const push = (k: string, v: number | null | undefined) => { const a = (hist.current[k] ??= []); a.push(v ?? 0); if (a.length > 60) a.shift(); };
    push("fps", stats.capture?.fps); push("encFps", (stats.recording ?? stats.replay)?.encoderFps); push("cpu", stats.system.appCpuPct);
    push("gpuEnc", stats.system.encoderPct); push("write", (stats.recording?.writeKbps ?? 0) / 1024); push("mem", stats.system.appMem / 1048576);
    force((n) => n + 1);
  }, [stats]);

  const na = t("common.na");
  const fmt = (v: number | null | undefined, d = 0) => (v == null || Number.isNaN(v) ? na : v.toFixed(d));
  const s: StatsPayload | null = stats;
  const sink = s?.recording ?? s?.replay ?? null;
  const h = hist.current;
  return (
    <div className="page">
      <h1 className="page-title">{t("nav.diagnostics")}</h1>
      <p className="dim" style={{ marginBottom: 16, maxWidth: 720 }}>{t("diag.intro")}</p>
      <div className="chips" style={{ marginBottom: 18 }}>
        <span className={`chip ${status.state === "recording" ? "danger" : status.replay ? "ok" : ""}`}><Activity size={13} />{status.state === "recording" ? t("rec.recording") : status.replay ? t("replay.on") : t("diag.idle")}</span>
        {!s && <span className="chip">{t("common.loading")}</span>}
      </div>

      <h2 className="section-title first">{t("diag.capture")}</h2>
      <div className="diag-grid">
        <Tile label={t("diag.captureFps")} value={fmt(s?.capture?.fps, 1)} unit=" /s" history={h.fps} hint={t("diag.captureFpsHint")} />
        <Tile label={t("diag.low1")} value={fmt(s?.capture?.low1, 1)} unit=" /s" />
        <Tile label={t("diag.resolution")} value={s?.capture ? `${s.capture.width}×${s.capture.height}` : na} />
        <Tile label={t("diag.dropped")} value={sink ? String(sink.droppedFrames) : na} unit={sink ? ` (${sink.droppedPct.toFixed(1)}%)` : ""} hint={t("diag.droppedHint")} />
        <Tile label={t("diag.skipped")} value={s?.capture ? String(s.capture.regulatorSkipped) : na} hint={t("diag.skippedHint")} />
      </div>

      <h2 className="section-title">{t("diag.encoder")}</h2>
      <div className="diag-grid">
        <Tile label={t("diag.encoderName")} value={sink?.encoder ?? na} />
        <Tile label={t("diag.encoderFps")} value={fmt(sink?.encoderFps, 1)} unit=" fps" history={h.encFps} />
        <Tile label={t("diag.speed")} value={sink ? sink.speed.toFixed(2) : na} unit="×" hint={t("diag.speedHint")} />
        <Tile label={t("diag.bitrate")} value={sink ? (sink.bitrateKbps / 1000).toFixed(1) : na} unit=" Mbps" />
        <Tile label={t("diag.gpuEncoder")} value={fmt(s?.system.encoderPct)} unit={s?.system.encoderPct != null ? "%" : ""} history={h.gpuEnc} />
        <Tile label={t("diag.diskWrite")} value={s?.recording ? (s.recording.writeKbps / 1024).toFixed(2) : na} unit=" MB/s" history={h.write} />
      </div>

      <h2 className="section-title">{t("diag.audio")}</h2>
      <div className="diag-grid">
        <Tile label={t("diag.audioLatency")} value={s ? String(s.audioLatencyMs || na) : na} unit={s?.audioLatencyMs ? " ms" : ""} hint={t("diag.audioLatencyHint")} />
        <Tile label={t("diag.lateTicks")} value={s ? String(s.audioLateTicks) : na} />
      </div>

      <h2 className="section-title">{t("diag.system")}</h2>
      <div className="diag-grid">
        <Tile label={t("diag.appCpu")} value={fmt(s?.system.appCpuPct, 1)} unit="%" history={h.cpu} hint={t("diag.appCpuHint")} />
        <Tile label={t("diag.appMem")} value={s ? formatBytes(s.system.appMem, 0) : na} history={h.mem} />
        <Tile label={t("diag.cpu")} value={fmt(s?.system.cpuPct)} unit="%" />
        <Tile label={t("diag.gpu")} value={fmt(s?.system.gpuPct)} unit={s?.system.gpuPct != null ? "%" : ""} />
        <Tile label={t("diag.gpuTemp")} value={fmt(s?.system.gpuTempC)} unit={s?.system.gpuTempC != null ? "°C" : ""} />
        <Tile label={t("diag.gpuMem")} value={s?.system.gpuMemUsed != null ? formatBytes(s.system.gpuMemUsed, 1) : na} />
        <Tile label={t("diag.ram")} value={s ? `${formatBytes(s.system.ramUsed, 0)} / ${formatBytes(s.system.ramTotal, 0)}` : na} />
      </div>
      <InfoBanner>{t("diag.gpuNote")}</InfoBanner>

      <h2 className="section-title">{t("diag.history")}</h2>
      {sessions.length === 0 ? <p className="faint">{t("diag.noHistory")}</p> : (
        <div className="card" style={{ padding: "4px 20px" }}>
          {sessions.map((r) => (
            <div key={r.id} className="field"><div className="field-text"><div className="field-label">{r.game || t("library.desktop")}</div><div className="field-hint num">{formatDate(r.startedAt, lang)}</div></div>
              <code className="path" style={{ maxWidth: 380 }} title={r.finalPath}>{r.finalPath}</code></div>
          ))}
        </div>
      )}
    </div>
  );
}

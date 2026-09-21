import { useCallback, useEffect, useMemo, useRef, useState, type ReactNode } from "react";
import { AnimatePresence, motion } from "motion/react";
import { Activity, AppWindow, Camera, Check, Clapperboard, Cpu, Crop, Gamepad2, Gauge, HardDrive, Layers, Maximize2, Mic, MicOff, Minimize2, Monitor, MousePointer2, RotateCcw, Save, Settings as SettingsIcon, Square, Video, Volume2, VolumeX, Zap } from "lucide-react";
import { Logo } from "@/components/Logo";
import { Meter } from "@/components/feedback";
import { KeyCaps } from "@/components/controls";
import { useT } from "@/i18n";
import { useSettings } from "@/stores/settings";
import { useDevices } from "@/stores/devices";
import { useRecording, elapsedMs, isRecording } from "@/stores/recording";
import { useLibrary } from "@/stores/library";
import { useLevels, useNow, useStats, useTauriEvent } from "@/hooks/hooks";
import { actions } from "@/features/common/actions";
import { ipc, on } from "@/lib/ipc";
import { formatBytes, formatClock, resolutionLabel } from "@/lib/format";
import { estimateBytes, PRESETS, recommendedBitrate } from "@/lib/encoders";
import type { Settings, StorageInfo } from "@/lib/types";
import { COMPACT_MODULES } from "./modules";
import { useGamepadNav, useSpatialNav } from "./nav";
import { ClipThumb } from "@/features/library/ClipCard";

function Tile({ icon, label, sub, onClick, active, danger, wide, children, hotkey, disabled }: { icon: ReactNode; label: string; sub?: ReactNode; onClick: () => void; active?: boolean; danger?: boolean; wide?: boolean; children?: ReactNode; hotkey?: string; disabled?: boolean }) {
  return (
    <button data-nav className={`ov-tile${active ? " on" : ""}${danger ? " danger" : ""}${wide ? " wide" : ""}`} onClick={onClick} disabled={disabled} aria-pressed={active}>
      <span className="ov-tile-ico">{icon}</span>
      <span className="ov-tile-text"><span className="ov-tile-label">{label}</span>{sub && <span className="ov-tile-sub">{sub}</span>}</span>
      {hotkey && <span className="ov-tile-key"><KeyCaps accel={hotkey} /></span>}
      {children}
    </button>
  );
}

function Panel({ title, icon, children, span = 1 }: { title: string; icon: ReactNode; children: ReactNode; span?: number }) {
  return <section className="ov-module" style={{ gridColumn: `span ${span}` }}><div className="ov-module-title">{icon}{title}</div>{children}</section>;
}

export function Overlay() {
  const t = useT();
  const s = useSettings((x) => x.settings);
  const patch = useSettings((x) => x.patch);
  const status = useRecording((x) => x.status);
  const monitors = useDevices((d) => d.monitors);
  const profiles = useDevices((d) => d.profiles);
  const clips = useLibrary((l) => l.clips);
  const refreshClips = useLibrary((l) => l.refresh);
  const [open, setOpen] = useState(false);
  const [openKey, setOpenKey] = useState(0);
  const [flash, setFlash] = useState(false);
  const [saveStage, setSaveStage] = useState<"idle" | "saving" | "done">("idle");
  const [storage, setStorage] = useState<StorageInfo | null>(null);
  const root = useRef<HTMLDivElement>(null);
  const rec = isRecording(status);
  const now = useNow(open && status.state === "recording", 250);
  const stats = useStats(open);
  const lv = useLevels(open);

  const close = useCallback(() => { setOpen(false); window.setTimeout(() => void ipc.overlayHide(), 130); }, []);

  useTauriEvent("overlay-shown", () => {
    setOpen(true); setOpenKey((k) => k + 1);
    void useDevices.getState().loadMonitors(); void useDevices.getState().loadProfiles(); void refreshClips();
    void ipc.storageInfo().then(setStorage).catch(() => undefined);
    void useRecording.getState().refresh();
  });
  useTauriEvent("overlay-hidden", () => setOpen(false));
  useTauriEvent("screenshot-flash", () => { setFlash(true); window.setTimeout(() => setFlash(false), 420); });
  useEffect(() => { const p = on<{ stage: string }>("replay-save", (e) => { if (e.stage === "saving") setSaveStage("saving"); else { setSaveStage(e.stage === "done" ? "done" : "idle"); window.setTimeout(() => setSaveStage("idle"), 1800); } }); return () => void p.then((u) => u()); }, []);

  useEffect(() => {
    if (!open) return;
    const k = (e: KeyboardEvent) => { if (e.key === "Escape") close(); };
    window.addEventListener("keydown", k);
    const id = window.setTimeout(() => root.current?.querySelector<HTMLElement>("[data-nav]")?.focus(), 60);
    return () => { window.removeEventListener("keydown", k); window.clearTimeout(id); };
  }, [open, openKey, close]);
  useSpatialNav(root, open);
  useGamepadNav(root, open, close);

  const profile = profiles.find((p) => p.id === s?.activeProfileId);
  const mods = useMemo(() => (s ? (s.overlay.mode === "compact" ? COMPACT_MODULES : (s.overlay.modules as string[])) : []), [s]);
  if (!s) return null;
  const show = (m: string) => mods.includes(m);
  const compact = s.overlay.mode === "compact";
  const hk = s.hotkeys;
  const scale = { small: 0.86, medium: 1, large: 1.18 }[s.overlay.scale];
  const setMode = (mode: "compact" | "full") => patch((d) => { d.overlay.mode = mode; }, { immediate: true });
  const applyPreset = async (name: string) => {
    if (!profile) return;
    const p = PRESETS[name];
    await ipc.saveProfile({ ...profile, quality: { ...profile.quality, ...p, preset: name as never, bitrateKbps: recommendedBitrate(p.height, p.fps, profile.quality.codec) * (name === "ultra" ? 2 : 1) } });
    await useDevices.getState().loadProfiles();
  };
  const modes: { v: Settings["capture"]["mode"]; label: string; icon: ReactNode }[] = [
    { v: "display", label: t("source.display"), icon: <Monitor size={15} /> }, { v: "window", label: t("source.window"), icon: <AppWindow size={15} /> },
    { v: "game", label: t("source.game"), icon: <Gamepad2 size={15} /> }, { v: "region", label: t("source.region"), icon: <Crop size={15} /> }, { v: "active", label: t("source.active"), icon: <MousePointer2 size={15} /> },
  ];

  const timer = formatClock(rec ? elapsedMs(status, now) : 0);
  const source = s.capture.mode === "display" ? t("source.displayN", { n: s.capture.displayIndex }) : s.capture.mode === "game" ? status.game?.name ?? t("source.game") : s.capture.mode === "window" ? s.capture.windowTitle || t("source.window") : s.capture.mode === "region" ? `${s.capture.region.w}×${s.capture.region.h}` : t("source.active");

  return (
    <div className="ov-root" onMouseDown={(e) => e.target === e.currentTarget && close()}>
      <AnimatePresence>
        {open && (
          <motion.div key={openKey} ref={root} className={`ov-panel${compact ? " compact" : ""}`} style={{ ["--ov-scale" as string]: scale }}
            initial={{ opacity: 0, scale: 0.94, filter: "blur(10px)", y: 14 }} animate={{ opacity: 1, scale: 1, filter: "blur(0px)", y: 0 }} exit={{ opacity: 0, scale: 0.97, filter: "blur(6px)" }}
            transition={{ type: "spring", stiffness: 460, damping: 34, mass: 0.8 }} role="dialog" aria-label="Rimlight">
            <AnimatePresence>{flash && <motion.div className="ov-flash" initial={{ opacity: 0.7 }} animate={{ opacity: 0 }} exit={{ opacity: 0 }} transition={{ duration: 0.4 }} />}</AnimatePresence>

            <header className="ov-head">
              <div className="ov-brand"><Logo size={26} recording={rec} /><span>Rimlight</span></div>
              <div className="ov-status">
                <span className={`chip ${status.state === "recording" ? "danger" : status.state === "paused" ? "warn" : ""}`}><span className={`dot ${status.state === "recording" ? "rec" : ""}`} /><span className="mono num">{rec ? timer : t("rec.ready")}</span></span>
                {!compact && <span className="chip"><Monitor size={13} />{source}</span>}
                {stats?.capture && !compact && <span className="chip"><Activity size={13} /><span className="num">{Math.round(stats.capture.fps)} fps</span></span>}
                {!compact && (status.recording?.encoder || status.replay?.encoder) && <span className="chip accent"><Zap size={13} />{(status.recording?.encoder ?? status.replay?.encoder ?? "").replace(/^(NVIDIA|AMD|Intel) /, "")}</span>}
                <span className={`chip ${s.audio.micEnabled ? "ok" : ""}`}>{s.audio.micEnabled ? <Mic size={13} /> : <MicOff size={13} />}{s.audio.micEnabled ? t("overlay.micOn") : t("overlay.micOff")}</span>
                {!compact && storage && <span className="chip"><HardDrive size={13} /><span className="num">{formatBytes(storage.freeBytes, 0)}</span></span>}
              </div>
              <div className="ov-head-actions">
                <button className="btn icon sm ghost" aria-label={compact ? t("overlay.full") : t("overlay.compact")} onClick={() => setMode(compact ? "full" : "compact")}>{compact ? <Maximize2 size={16} /> : <Minimize2 size={16} />}</button>
                <button className="btn sm ghost" onClick={close}>Esc</button>
              </div>
            </header>

            <div className="ov-tiles">
              {show("record") && <Tile wide={!compact} active={rec} danger={rec} icon={rec ? <Square size={20} fill="currentColor" /> : <Video size={21} />} label={rec ? t("rec.stop") : t("rec.start")}
                sub={rec ? <span className="mono num">{timer}</span> : t("rec.startHint")} hotkey={hk.record} disabled={status.state === "preparing" || status.state === "stopping"} onClick={() => void actions.toggleRecord()} />}
              {show("replay") && <Tile active={!!status.replay} icon={<RotateCcw size={20} />} label={t("replay.title")} sub={status.replay ? t("replay.on") : t("replay.off")} hotkey={hk.toggleReplay} onClick={() => void actions.toggleReplay()} />}
              {show("saveReplay") && (
                <Tile disabled={!status.replay || status.replay.saving} icon={saveStage === "done" ? <Check size={20} /> : <Save size={20} />} label={t("replay.save")} hotkey={hk.saveReplay}
                  sub={saveStage === "saving" ? t("replay.saving") : saveStage === "done" ? t("replay.saved") : status.replay ? t("replay.lastN", { n: s.replay.durationSecs >= 60 ? `${s.replay.durationSecs / 60}m` : `${s.replay.durationSecs}s` }) : t("replay.needsOn")} onClick={() => void actions.saveReplay()}>
                  {saveStage === "saving" && <span className="ov-tile-progress"><i /></span>}
                </Tile>)}
              {show("screenshot") && <Tile icon={<Camera size={20} />} label={t("shot.take")} sub={t("shot.hint")} hotkey={hk.screenshot} onClick={() => void actions.screenshot()} />}
              {show("mic") && (
                <Tile active={s.audio.micEnabled} icon={<motion.span key={String(s.audio.micEnabled)} initial={{ scale: 0.6, rotate: -20 }} animate={{ scale: 1, rotate: 0 }} transition={{ type: "spring", stiffness: 500, damping: 18 }} style={{ display: "grid" }}>{s.audio.micEnabled ? <Mic size={20} /> : <MicOff size={20} />}</motion.span>}
                  label={t("audio.mic")} sub={s.audio.micEnabled ? t("overlay.micOn") : t("overlay.micOff")} hotkey={hk.toggleMic} onClick={() => actions.toggleMic()}>
                  <span className="ov-tile-meter"><Meter level={lv.mic} off={!s.audio.micEnabled} label={t("mic.level")} /></span>
                </Tile>)}
              {show("system") && (
                <Tile active={s.audio.systemEnabled} icon={s.audio.systemEnabled ? <Volume2 size={20} /> : <VolumeX size={20} />} label={t("audio.system")} sub={s.audio.systemEnabled ? `${s.audio.systemVolume}%` : t("overlay.micOff")} onClick={() => patch((d) => { d.audio.systemEnabled = !d.audio.systemEnabled; }, { immediate: true })}>
                  <span className="ov-tile-meter"><Meter level={lv.system} off={!s.audio.systemEnabled} label={t("audio.system")} /></span>
                </Tile>)}
              {show("camera") && <Tile active={s.camera.enabled} icon={<Camera size={20} />} label={t("camera.title")} sub={s.camera.enabled ? (s.camera.device || t("camera.none")) : t("overlay.micOff")} hotkey={hk.toggleCamera} onClick={() => actions.toggleCamera()} />}
              {show("stats") && <Tile active={s.performance.statsEnabled} icon={<Gauge size={20} />} label={t("perf.stats")} sub={s.performance.statsEnabled ? t("overlay.micOn") : t("overlay.micOff")} hotkey={hk.toggleStats} onClick={() => patch((d) => { d.performance.statsEnabled = !d.performance.statsEnabled; }, { immediate: true })} />}
            </div>

            {!compact && (
              <div className="ov-modules">
                {show("mode") && (
                  <Panel title={t("source.title")} icon={<Layers size={14} />} span={show("source") ? 1 : 2}>
                    <div className="ov-seg" role="group" aria-label={t("source.title")}>{modes.map((m) => <button key={m.v} data-nav aria-pressed={s.capture.mode === m.v} onClick={() => patch((d) => { d.capture.mode = m.v; }, { immediate: true })}>{m.icon}<span>{m.label}</span></button>)}</div>
                    {s.capture.mode === "region" && <button data-nav className="btn sm" style={{ marginTop: 10 }} onClick={() => { void ipc.overlayHide(); void ipc.regionPick(); }}><Crop size={14} />{t("source.selectRegion")}</button>}
                  </Panel>)}
                {show("source") && (
                  <Panel title={t("source.display")} icon={<Monitor size={14} />}>
                    <div className="ov-monitors">{monitors.map((m) => (
                      <button key={m.index} data-nav className="ov-monitor" aria-pressed={s.capture.displayIndex === m.index && s.capture.mode === "display"} onClick={() => patch((d) => { d.capture.displayIndex = m.index; d.capture.mode = "display"; }, { immediate: true })}>
                        <span className="num ov-monitor-n">{m.index}</span><span><b>{t("source.displayN", { n: m.index })}</b>{m.primary && <i className="ov-primary">{t("source.primary")}</i>}<small className="num">{m.width}×{m.height} · {m.refreshHz} Hz</small></span></button>))}</div>
                  </Panel>)}
                {show("mixer") && (
                  <Panel title={t("audio.mixer")} icon={<Volume2 size={14} />}>
                    {([["system", t("audio.system"), s.audio.systemVolume, lv.system, s.audio.systemEnabled], ["mic", t("audio.mic"), s.audio.micVolume, lv.mic, s.audio.micEnabled]] as const).map(([k, label, vol, level, on]) => (
                      <div key={k} className="ov-mixer">
                        <div className="ov-mixer-head"><span>{label}</span><span className="mono num dim">{vol}%</span></div>
                        <Meter level={level} off={!on} label={label} />
                        <input data-nav type="range" className="slider" aria-label={label} min={0} max={200} value={vol} style={{ ["--p" as string]: `${vol / 2}%` }} onChange={(e) => patch((d) => { if (k === "system") d.audio.systemVolume = Number(e.target.value); else d.audio.micVolume = Number(e.target.value); })} />
                      </div>))}
                  </Panel>)}
                {show("quality") && profile && (
                  <Panel title={t("quality.title")} icon={<Zap size={14} />}>
                    <div className="ov-seg" role="group" aria-label={t("quality.preset")}>{(["low", "medium", "high", "ultra"] as const).map((p) => <button key={p} data-nav aria-pressed={profile.quality.preset === p} onClick={() => void applyPreset(p)}><span>{t(`quality.${p}` as "quality.low")}</span></button>)}</div>
                    <div className="dim num" style={{ fontSize: 12.5, marginTop: 10 }}>{resolutionLabel(profile.quality.height)} · {profile.quality.fps} fps · {profile.quality.codec.toUpperCase()} · {(profile.quality.bitrateKbps / 1000).toFixed(0)} Mbps</div>
                    <div className="faint num" style={{ fontSize: 12 }}>{t("quality.estimate", { size: formatBytes(estimateBytes(profile.quality, 3, 10), 0), min: 10 })}</div>
                  </Panel>)}
                {show("clips") && (
                  <Panel title={t("home.recent")} icon={<Clapperboard size={14} />} span={2}>
                    {clips.length === 0 ? <div className="faint" style={{ fontSize: 13 }}>{t("home.noClips")}</div> : (
                      <div className="ov-clips">{clips.slice(0, 5).map((c) => <button key={c.id} data-nav className="ov-clip" aria-label={c.title} onClick={() => void ipc.openPath(c.path).then(close).catch(() => undefined)}><ClipThumb clip={c} /></button>)}</div>)}
                  </Panel>)}
              </div>
            )}

            <footer className="ov-foot">
              <span className="faint"><Cpu size={12} style={{ verticalAlign: -2 }} /> {stats?.system ? <span className="num">CPU {Math.round(stats.system.cpuPct)}%{stats.system.gpuPct != null ? ` · GPU ${stats.system.gpuPct}%` : ""}</span> : t("overlay.navHint")}</span>
              <span className="row-gap">
                {show("settings") && <button data-nav className="btn sm ghost" onClick={() => void ipc.openMain("settings")}><SettingsIcon size={14} />{t("nav.settings")}</button>}
                <button data-nav className="btn sm ghost" onClick={() => void ipc.openMain("home")}>{t("overlay.openApp")}</button>
              </span>
            </footer>
          </motion.div>
        )}
      </AnimatePresence>
    </div>
  );
}

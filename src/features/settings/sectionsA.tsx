import { useCallback, useEffect, useState, type ReactNode } from "react";
import { open } from "@tauri-apps/plugin-dialog";
import { Download, FolderOpen, RefreshCw } from "lucide-react";
import { Field, NumberInput, Segmented, Select, Slider, Toggle } from "@/components/controls";
import { Chip, Meter } from "@/components/feedback";
import { ErrorBanner, InfoBanner } from "@/components/overlays";
import { QualityEditor } from "@/features/profiles/QualityEditor";
import { useT } from "@/i18n";
import { useSettings } from "@/stores/settings";
import { useDevices } from "@/stores/devices";
import { useLevels, useVisible } from "@/hooks/hooks";
import { ipc, normalizeError } from "@/lib/ipc";
import type { AppError, Settings, UpdateInfo } from "@/lib/types";
import { attempt } from "@/features/common/actions";
import { REPLAY_DURATIONS, durationLabel } from "@/features/home/Cards";

export function useS() {
  const s = useSettings((x) => x.settings)!;
  const patch = useSettings((x) => x.patch);
  const set = (mutate: (d: Settings) => void) => patch(mutate, { immediate: true });
  return { s, patch, set };
}

export const Card = ({ children }: { children: ReactNode }) => <div className="card settings-card">{children}</div>;

export async function pickFolder(current: string): Promise<string | null> {
  const r = await open({ directory: true, multiple: false, defaultPath: current || undefined });
  return typeof r === "string" ? r : null;
}

export function General() {
  const t = useT();
  const { s, set, patch } = useS();
  const [upd, setUpd] = useState<UpdateInfo | null>(null);
  const [updErr, setUpdErr] = useState<AppError | null>(null);
  const [busy, setBusy] = useState(false);
  const check = async () => {
    setBusy(true); setUpdErr(null);
    try { setUpd(await ipc.checkUpdate()); } catch (e) { setUpdErr(normalizeError(e)); } finally { setBusy(false); }
  };
  return (
    <Card>
      <Field label={t("general.launchWithWindows")} hint={t("general.launchHint")}><Toggle label={t("general.launchWithWindows")} checked={s.general.launchWithWindows} onChange={(v) => set((d) => { d.general.launchWithWindows = v; })} /></Field>
      <Field label={t("general.startMode")} hint={t("general.startModeHint")}>
        <Select label={t("general.startMode")} value={s.general.startMode} onChange={(v) => set((d) => { d.general.startMode = v; })}
          options={[{ value: "normal", label: t("general.startNormal") }, { value: "minimized", label: t("general.startMinimized") }, { value: "tray", label: t("general.startTray") }]} />
      </Field>
      <Field label={t("general.closeToTray")} hint={t("general.closeToTrayHint")}><Toggle label={t("general.closeToTray")} checked={s.general.closeToTray} onChange={(v) => set((d) => { d.general.closeToTray = v; })} /></Field>
      <Field label={t("general.autoReplay")} hint={t("general.autoReplayHint")}><Toggle label={t("general.autoReplay")} checked={s.general.autoStartReplay} onChange={(v) => set((d) => { d.general.autoStartReplay = v; })} /></Field>
      <Field label={t("general.hwAccel")} hint={t("general.hwAccelHint")}><Toggle label={t("general.hwAccel")} checked={s.general.hardwareAcceleration} onChange={(v) => set((d) => { d.general.hardwareAcceleration = v; })} /></Field>
      <Field label={t("general.checkUpdates")} hint={t("general.updatesHint")}><Toggle label={t("general.checkUpdates")} checked={s.general.checkUpdates} onChange={(v) => set((d) => { d.general.checkUpdates = v; })} /></Field>
      <Field label={t("general.feedUrl")} hint={t("general.feedHint")} wide>
        <input className="input mono" dir="ltr" aria-label={t("general.feedUrl")} placeholder="https://example.com/rimlight/latest.json" value={s.general.updateFeedUrl} onChange={(e) => patch((d) => { d.general.updateFeedUrl = e.target.value; })} />
        <button className="btn" disabled={busy || !s.general.updateFeedUrl} onClick={() => void check()}><RefreshCw size={15} className={busy ? "spin" : ""} />{t("general.checkNow")}</button>
      </Field>
      {updErr && <ErrorBanner error={updErr} onDismiss={() => setUpdErr(null)} />}
      {upd && (
        <div className="banner" style={{ marginTop: 10 }}>
          <Download size={18} className="accent-text" />
          <div style={{ flex: 1 }}>
            <div className="b-title">{upd.available ? t("general.updateAvailable", { version: upd.latest }) : t("general.upToDate", { version: upd.current })}</div>
            {upd.available && upd.notes && <div className="b-body" style={{ whiteSpace: "pre-wrap" }}>{upd.notes}</div>}
          </div>
          {upd.available && <button className="btn primary sm" onClick={() => void attempt(ipc.installUpdate(upd.url))}>{t("general.install")}</button>}
        </div>
      )}
    </Card>
  );
}

export function Recording() {
  const t = useT();
  const { s, set, patch } = useS();
  const profiles = useDevices((d) => d.profiles);
  const load = useDevices((d) => d.loadProfiles);
  const profile = profiles.find((p) => p.id === s.activeProfileId);
  useEffect(() => { void load(); }, [load]);
  return (
    <>
      <Card>
        <Field label={t("rec.cursor")}><Toggle label={t("rec.cursor")} checked={s.capture.captureCursor} onChange={(v) => set((d) => { d.capture.captureCursor = v; })} /></Field>
        <Field label={t("rec.highlightCursor")} hint={t("rec.highlightHint")}><Toggle label={t("rec.highlightCursor")} checked={s.capture.highlightCursor} onChange={(v) => set((d) => { d.capture.highlightCursor = v; })} /></Field>
        <Field label={t("rec.showClicks")} hint={t("rec.showClicksHint")}><Toggle label={t("rec.showClicks")} checked={s.capture.showClicks} onChange={(v) => set((d) => { d.capture.showClicks = v; })} /></Field>
        <Field label={t("rec.stopWhenClosed")} hint={t("rec.stopWhenClosedHint")}><Toggle label={t("rec.stopWhenClosed")} checked={s.capture.stopWhenAppCloses} onChange={(v) => set((d) => { d.capture.stopWhenAppCloses = v; })} /></Field>
        <Field label={t("rec.fileTemplate")} hint={t("rec.fileTemplateHint")} wide>
          <input className="input mono" dir="ltr" aria-label={t("rec.fileTemplate")} value={s.fileNameTemplate} onChange={(e) => patch((d) => { d.fileNameTemplate = e.target.value; })} />
        </Field>
      </Card>
      <h2 className="section-title">{t("rec.qualityFor", { name: profile?.name ?? "" })}</h2>
      {profile && <Card><QualityEditor q={profile.quality} onChange={(q) => void ipc.saveProfile({ ...profile, quality: q }).then(() => ipc.setActiveProfile(profile.id)).then((ns) => { useSettings.getState().replace(ns); void load(); })} /></Card>}
    </>
  );
}

export function Replay() {
  const t = useT();
  const { s, set } = useS();
  const custom = !REPLAY_DURATIONS.includes(s.replay.durationSecs);
  return (
    <Card>
      <Field label={t("replay.length")} hint={t("replay.lengthHint")}>
        <Select label={t("replay.length")} value={custom ? -1 : s.replay.durationSecs} onChange={(v) => set((d) => { d.replay.durationSecs = v < 0 ? 90 : v; })}
          options={[...REPLAY_DURATIONS.map((d) => ({ value: d, label: durationLabel(t, d) })), { value: -1, label: t("replay.custom") }]} />
      </Field>
      {custom && <Field label={t("replay.customSeconds")}><NumberInput label={t("replay.customSeconds")} min={10} max={3600} value={s.replay.durationSecs} onChange={(v) => set((d) => { d.replay.durationSecs = v; })} /></Field>}
      <Field label={t("general.autoReplay")} hint={t("general.autoReplayHint")}><Toggle label={t("general.autoReplay")} checked={s.general.autoStartReplay} onChange={(v) => set((d) => { d.general.autoStartReplay = v; })} /></Field>
      <InfoBanner>{t("replay.howItWorks")}</InfoBanner>
    </Card>
  );
}

export function AudioSection() {
  const t = useT();
  const { s, set, patch } = useS();
  const dev = useDevices((d) => d.audio);
  const load = useDevices((d) => d.loadAudio);
  const visible = useVisible();
  const lv = useLevels(visible);
  useEffect(() => { void load(); }, [load]);
  const outs = [{ value: "", label: t("audio.defaultDevice") }, ...dev.outputs.map((d) => ({ value: d.name, label: d.name }))];
  return (
    <Card>
      <Field label={t("audio.system")} hint={t("audio.systemHint")}><Toggle label={t("audio.system")} checked={s.audio.systemEnabled} onChange={(v) => set((d) => { d.audio.systemEnabled = v; })} /></Field>
      <Field label={t("audio.outputDevice")} wide><Select label={t("audio.outputDevice")} value={s.audio.systemDevice} options={outs} onChange={(v) => set((d) => { d.audio.systemDevice = v; })} /><button className="btn icon" aria-label={t("common.refresh")} onClick={() => void load()}><RefreshCw size={15} /></button></Field>
      <Field label={t("audio.systemVolume")}><Slider label={t("audio.systemVolume")} value={s.audio.systemVolume} min={0} max={200} onChange={(v) => patch((d) => { d.audio.systemVolume = v; })} format={(v) => `${v}%`} /></Field>
      <div style={{ padding: "0 0 14px" }}><Meter level={lv.system} off={!s.audio.systemEnabled} label={t("audio.system")} /></div>
      <Field label={t("audio.tracks")} hint={t("audio.tracksHint")}>
        <div className="chips"><Chip tone="accent">{t("audio.track1")}</Chip>{s.audio.trackSystem && <Chip>{t("audio.track2")}</Chip>}{s.audio.trackMic && <Chip>{t("audio.track3")}</Chip>}</div>
      </Field>
      <Field label={t("audio.trackSystem")}><Toggle label={t("audio.trackSystem")} checked={s.audio.trackSystem} onChange={(v) => set((d) => { d.audio.trackSystem = v; })} /></Field>
      <Field label={t("audio.trackMic")}><Toggle label={t("audio.trackMic")} checked={s.audio.trackMic} onChange={(v) => set((d) => { d.audio.trackMic = v; })} /></Field>
      <Field label={t("audio.fallback")} hint={t("audio.fallbackHint")}><Toggle label={t("audio.fallback")} checked={s.audio.fallbackToDefault} onChange={(v) => set((d) => { d.audio.fallbackToDefault = v; })} /></Field>
      <Field label={t("audio.perApp")} hint={t("audio.perAppHint")}><Chip tone="warn">{t("common.comingSoon")}</Chip></Field>
    </Card>
  );
}

function KeyCapture({ value, onChange, label }: { value: string; onChange: (v: string) => void; label: string }) {
  const t = useT();
  const [cap, setCap] = useState(false);
  useEffect(() => {
    if (!cap) return;
    const kd = (e: KeyboardEvent) => {
      e.preventDefault(); e.stopPropagation();
      if (e.key === "Escape") return setCap(false);
      let name: string | null = null;
      if (/^Key[A-Z]$/.test(e.code)) name = e.code.slice(3);
      else if (/^Digit\d$/.test(e.code)) name = e.code.slice(5);
      else if (/^F\d+$/.test(e.code)) name = e.code;
      else if (e.code === "Space") name = "Space";
      else if (e.code === "Tab") name = "Tab";
      else if (e.code === "CapsLock") name = "CapsLock";
      else if (e.code === "ControlLeft") name = "LCtrl";
      else if (e.code === "AltLeft") name = "LAlt";
      else if (e.code === "ShiftLeft") name = "LShift";
      if (name) { onChange(name); setCap(false); }
    };
    const md = (e: MouseEvent) => {
      const n = e.button === 1 ? "MiddleMouse" : e.button === 3 ? "Mouse4" : e.button === 4 ? "Mouse5" : null;
      if (n) { e.preventDefault(); onChange(n); setCap(false); }
    };
    window.addEventListener("keydown", kd, true); window.addEventListener("mousedown", md, true);
    return () => { window.removeEventListener("keydown", kd, true); window.removeEventListener("mousedown", md, true); };
  }, [cap, onChange]);
  return <button className={`btn${cap ? " primary" : ""}`} aria-label={label} onClick={() => setCap(!cap)}>{cap ? t("hotkeys.press") : <kbd className="keycap">{value}</kbd>}</button>;
}

export function Microphone() {
  const t = useT();
  const { s, set, patch } = useS();
  const dev = useDevices((d) => d.audio);
  const load = useDevices((d) => d.loadAudio);
  const visible = useVisible();
  const lv = useLevels(visible);
  useEffect(() => { void load(); }, [load]);
  const ins = [{ value: "", label: t("audio.defaultDevice") }, ...dev.inputs.map((d) => ({ value: d.name, label: d.name }))];
  const a = s.audio;
  return (
    <Card>
      <Field label={t("audio.mic")} hint={t("mic.enableHint")}><Toggle label={t("audio.mic")} checked={a.micEnabled} onChange={(v) => set((d) => { d.audio.micEnabled = v; })} /></Field>
      <Field label={t("mic.device")} wide><Select label={t("mic.device")} value={a.micDevice} options={ins} onChange={(v) => set((d) => { d.audio.micDevice = v; })} /><button className="btn icon" aria-label={t("common.refresh")} onClick={() => void load()}><RefreshCw size={15} /></button></Field>
      <div style={{ padding: "10px 0 4px" }}><Meter level={lv.mic} off={!a.micEnabled} label={t("mic.level")} /><div className="faint" style={{ fontSize: 12, marginTop: 6 }}>{a.micEnabled ? t("mic.levelHint") : t("mic.levelOff")}</div></div>
      <Field label={t("mic.volume")}><Slider label={t("mic.volume")} value={a.micVolume} min={0} max={200} onChange={(v) => patch((d) => { d.audio.micVolume = v; })} format={(v) => `${v}%`} /></Field>
      <Field label={t("mic.gain")}><Slider label={t("mic.gain")} value={a.micGainDb} min={-20} max={30} step={0.5} onChange={(v) => patch((d) => { d.audio.micGainDb = v; })} format={(v) => `${v > 0 ? "+" : ""}${v} dB`} /></Field>
      <Field label={t("mic.gate")} hint={t("mic.gateHint")}><Toggle label={t("mic.gate")} checked={a.gateEnabled} onChange={(v) => set((d) => { d.audio.gateEnabled = v; })} /></Field>
      {a.gateEnabled && <Field label={t("mic.gateThreshold")}><Slider label={t("mic.gateThreshold")} value={a.gateThresholdDb} min={-80} max={-10} onChange={(v) => patch((d) => { d.audio.gateThresholdDb = v; })} format={(v) => `${v} dB`} /></Field>}
      <Field label={t("mic.compressor")} hint={t("mic.compressorHint")}><Toggle label={t("mic.compressor")} checked={a.compressor} onChange={(v) => set((d) => { d.audio.compressor = v; })} /></Field>
      <Field label={t("mic.limiter")} hint={t("mic.limiterHint")}><Toggle label={t("mic.limiter")} checked={a.limiter} onChange={(v) => set((d) => { d.audio.limiter = v; })} /></Field>
      <Field label={t("mic.noise")} hint={t("mic.noiseHint")}><Chip tone="warn">{t("common.comingSoon")}</Chip></Field>
      <Field label={t("mic.ptt")} hint={t("mic.pttHint")}>
        <Segmented<Settings["audio"]["pttMode"]> label={t("mic.ptt")} value={a.pttMode} onChange={(v) => set((d) => { d.audio.pttMode = v; })}
          options={[{ value: "off", label: t("mic.pttOff") }, { value: "push_to_talk", label: t("mic.pushTalk") }, { value: "push_to_mute", label: t("mic.pushMute") }]} />
      </Field>
      {a.pttMode !== "off" && <Field label={t("mic.pttKey")} hint={t("mic.pttKeyHint")}><KeyCapture label={t("mic.pttKey")} value={a.pttKey} onChange={(v) => set((d) => { d.audio.pttKey = v; })} /></Field>}
    </Card>
  );
}

export function CameraSection() {
  const t = useT();
  const { s, set, patch } = useS();
  const cams = useDevices((d) => d.cameras);
  const load = useDevices((d) => d.loadCameras);
  const [loading, setLoading] = useState(false);
  const refresh = useCallback(async () => { setLoading(true); try { await load(); } finally { setLoading(false); } }, [load]);
  useEffect(() => { void refresh(); }, [refresh]);
  const c = s.camera;
  return (
    <Card>
      <Field label={t("camera.enable")} hint={t("camera.enableHint")}><Toggle label={t("camera.enable")} checked={c.enabled} onChange={(v) => set((d) => { d.camera.enabled = v; })} /></Field>
      <Field label={t("camera.device")} wide>
        <Select label={t("camera.device")} value={c.device} placeholder={loading ? t("common.loading") : t("camera.none")} options={cams.map((n) => ({ value: n, label: n }))} onChange={(v) => set((d) => { d.camera.device = v; })} />
        <button className="btn icon" aria-label={t("common.refresh")} onClick={() => void refresh()}><RefreshCw size={15} className={loading ? "spin" : ""} /></button>
      </Field>
      <Field label={t("camera.resolution")}>
        <Select label={t("camera.resolution")} value={`${c.width}x${c.height}`} onChange={(v) => { const [w, h] = v.split("x").map(Number); set((d) => { d.camera.width = w; d.camera.height = h; }); }}
          options={["640x480", "1280x720", "1920x1080"].map((r) => ({ value: r, label: r.replace("x", "×") }))} />
      </Field>
      <Field label={t("camera.fps")}><Select label={t("camera.fps")} value={c.fps} onChange={(v) => set((d) => { d.camera.fps = v; })} options={[15, 24, 30, 60].map((f) => ({ value: f, label: `${f} fps` }))} /></Field>
      <Field label={t("camera.shape")}>
        <Segmented<Settings["camera"]["shape"]> label={t("camera.shape")} value={c.shape} onChange={(v) => set((d) => { d.camera.shape = v; })}
          options={[{ value: "circle", label: t("camera.circle") }, { value: "rounded", label: t("camera.rounded") }, { value: "square", label: t("camera.square") }]} />
      </Field>
      <Field label={t("camera.size")}><Slider label={t("camera.size")} value={c.sizePct} min={8} max={60} onChange={(v) => patch((d) => { d.camera.sizePct = v; })} format={(v) => `${v}%`} /></Field>
      <Field label={t("camera.corner")}>
        <div className="corner-picker" role="radiogroup" aria-label={t("camera.corner")}>
          {(["top-left", "top-right", "bottom-left", "bottom-right"] as const).map((k) => <button key={k} role="radio" aria-checked={c.corner === k} aria-label={k} className={`corner ${k}`} onClick={() => set((d) => { d.camera.corner = k; })} />)}
        </div>
      </Field>
      <Field label={t("camera.margin")}><Slider label={t("camera.margin")} value={c.marginPx} min={0} max={200} onChange={(v) => patch((d) => { d.camera.marginPx = v; })} format={(v) => `${v}px`} /></Field>
      <Field label={t("camera.mirror")}><Toggle label={t("camera.mirror")} checked={c.mirror} onChange={(v) => set((d) => { d.camera.mirror = v; })} /></Field>
      <InfoBanner>{t("camera.note")}</InfoBanner>
    </Card>
  );
}

export { FolderOpen };

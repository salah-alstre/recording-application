import { useEffect, useState } from "react";
import { FolderOpen, Plus, RotateCcw, ScrollText, X } from "lucide-react";
import { Field, KeyCaps, NumberInput, Segmented, Select, Slider, Toggle } from "@/components/controls";
import { Chip } from "@/components/feedback";
import { Dialog, InfoBanner } from "@/components/overlays";
import { useT } from "@/i18n";
import { ipc, on } from "@/lib/ipc";
import { eventToAccel, isAcceptable } from "@/lib/hotkey";
import type { AppInfo, Corner, HotkeyStatus, Settings, StorageInfo } from "@/lib/types";
import { formatBytes } from "@/lib/format";
import { useDevices } from "@/stores/devices";
import { DEFAULT_HOTKEYS, HOTKEY_ACTIONS, OVERLAY_MODULES, STAT_METRICS } from "@/features/overlay/modules";
import { Card, pickFolder, useS } from "./sectionsA";
import { LanguageSwitch } from "./LanguageSwitch";

export function Screenshots() {
  const t = useT();
  const { s, set, patch } = useS();
  const sc = s.screenshots;
  return (
    <Card>
      <Field label={t("shot.format")}><Segmented<Settings["screenshots"]["format"]> label={t("shot.format")} value={sc.format} onChange={(v) => set((d) => { d.screenshots.format = v; })} options={[{ value: "png", label: "PNG" }, { value: "jpeg", label: "JPEG" }, { value: "webp", label: "WebP" }]} /></Field>
      {sc.format === "jpeg" && <Field label={t("shot.quality")}><Slider label={t("shot.quality")} value={sc.quality} min={40} max={100} onChange={(v) => patch((d) => { d.screenshots.quality = v; })} format={(v) => `${v}%`} /></Field>}
      {sc.format === "webp" && <InfoBanner>{t("shot.webpNote")}</InfoBanner>}
      <Field label={t("shot.dir")} hint={t("shot.dirHint")} wide>
        <input className="input mono" dir="ltr" readOnly aria-label={t("shot.dir")} value={sc.dir || t("shot.dirDefault")} />
        <button className="btn" onClick={() => void pickFolder(sc.dir).then((p) => p && set((d) => { d.screenshots.dir = p; }))}><FolderOpen size={15} />{t("common.browse")}</button>
        {sc.dir && <button className="btn icon" aria-label={t("common.reset")} onClick={() => set((d) => { d.screenshots.dir = ""; })}><RotateCcw size={15} /></button>}
      </Field>
      <Field label={t("shot.cursor")} hint={t("shot.cursorHint")}><Toggle label={t("shot.cursor")} checked={sc.includeCursor} onChange={(v) => set((d) => { d.screenshots.includeCursor = v; })} /></Field>
      <Field label={t("shot.delay")} hint={t("shot.delayHint")}><Select label={t("shot.delay")} value={sc.delaySecs} onChange={(v) => set((d) => { d.screenshots.delaySecs = v; })} options={[0, 3, 5, 10].map((n) => ({ value: n, label: n === 0 ? t("shot.noDelay") : t("unit.seconds", { n }) }))} /></Field>
      <Field label={t("shot.clipboard")} hint={t("shot.clipboardHint")}><Toggle label={t("shot.clipboard")} checked={sc.copyToClipboard} onChange={(v) => set((d) => { d.screenshots.copyToClipboard = v; })} /></Field>
      <div className="row-gap" style={{ paddingTop: 12, flexWrap: "wrap" }}>
        {[["current", "shot.modeCurrent"], ["display", "shot.modeDisplay"], ["window", "shot.modeWindow"], ["region", "shot.modeRegion"]].map(([m, k]) => (
          <button key={m} className="btn" onClick={() => void ipc.takeScreenshot(m).catch(() => undefined)}>{t(k as "shot.modeCurrent")}</button>
        ))}
      </div>
    </Card>
  );
}

export function OverlaySection() {
  const t = useT();
  const { s, set } = useS();
  const o = s.overlay;
  const toggle = (m: string) => set((d) => { d.overlay.modules = d.overlay.modules.includes(m) ? d.overlay.modules.filter((x) => x !== m) : [...d.overlay.modules, m]; });
  return (
    <Card>
      <Field label={t("overlay.mode")} hint={t("overlay.modeHint")}><Segmented<"compact" | "full"> label={t("overlay.mode")} value={o.mode} onChange={(v) => set((d) => { d.overlay.mode = v; })} options={[{ value: "compact", label: t("overlay.compact") }, { value: "full", label: t("overlay.full") }]} /></Field>
      <Field label={t("overlay.scale")}><Segmented<"small" | "medium" | "large"> label={t("overlay.scale")} value={o.scale} onChange={(v) => set((d) => { d.overlay.scale = v; })} options={[{ value: "small", label: t("overlay.small") }, { value: "medium", label: t("overlay.medium") }, { value: "large", label: t("overlay.large") }]} /></Field>
      <Field label={t("overlay.open")}><KeyCaps accel={s.hotkeys.overlay ?? ""} /></Field>
      <div className="field" style={{ alignItems: "flex-start" }}>
        <div className="field-text"><div className="field-label">{t("overlay.modules")}</div><div className="field-hint">{t("overlay.modulesHint")}</div></div>
        <div className="module-grid">
          {OVERLAY_MODULES.map((m) => (
            <label key={m} className="check"><input type="checkbox" checked={o.modules.includes(m)} onChange={() => toggle(m)} /><span>{t(`overlay.mod.${m}` as "overlay.mod.record")}</span></label>
          ))}
        </div>
      </div>
      <button className="btn" onClick={() => void ipc.overlayToggle()}>{t("overlay.preview")}</button>
    </Card>
  );
}

export function Hotkeys() {
  const t = useT();
  const { s, set } = useS();
  const [status, setStatus] = useState<HotkeyStatus[]>([]);
  const [capturing, setCapturing] = useState<string | null>(null);
  const [msg, setMsg] = useState<string>("");
  useEffect(() => {
    void ipc.hotkeyStatus().then(setStatus);
    const p = on<HotkeyStatus[]>("hotkeys-status", setStatus);
    return () => void p.then((u) => u());
  }, []);
  useEffect(() => {
    if (!capturing) return;
    const kd = async (e: KeyboardEvent) => {
      e.preventDefault(); e.stopPropagation();
      if (e.key === "Escape") { setCapturing(null); setMsg(""); return; }
      if (e.key === "Backspace" || e.key === "Delete") { set((d) => { d.hotkeys[capturing] = ""; }); setCapturing(null); return; }
      const accel = eventToAccel(e);
      if (!accel) return;
      if (!isAcceptable(accel)) { setMsg(t("hotkeys.needModifier")); return; }
      const r = await ipc.checkHotkey(capturing, accel);
      if (!r.ok) { setMsg(r.problem === "duplicate" ? t("hotkeys.duplicate", { other: t(`hotkeys.action.${r.conflictsWith}` as "hotkeys.action.record") }) : r.problem === "in_use" ? t("hotkeys.inUse") : t("hotkeys.invalid")); return; }
      set((d) => { d.hotkeys[capturing] = accel; });
      setCapturing(null); setMsg("");
    };
    window.addEventListener("keydown", kd, true);
    return () => window.removeEventListener("keydown", kd, true);
  }, [capturing, set, t]);
  const problem = (a: string) => status.find((x) => x.action === a && !x.ok);
  return (
    <Card>
      {HOTKEY_ACTIONS.map((a) => (
        <div key={a} className="field">
          <div className="field-text"><div className="field-label">{t(`hotkeys.action.${a}` as "hotkeys.action.record")}</div>
            {problem(a) && <div className="field-hint" style={{ color: "var(--warn)" }}>{t(problem(a)!.problem === "duplicate" ? "hotkeys.duplicateShort" : problem(a)!.problem === "in_use" ? "hotkeys.inUse" : problem(a)!.problem === "empty" ? "hotkeys.disabled" : "hotkeys.invalid")}</div>}</div>
          <div className="field-control">
            <button className={`btn${capturing === a ? " primary" : ""}`} aria-label={t("hotkeys.change")} onClick={() => { setCapturing(capturing === a ? null : a); setMsg(""); }}>
              {capturing === a ? t("hotkeys.press") : s.hotkeys[a] ? <KeyCaps accel={s.hotkeys[a]} /> : <span className="dim">{t("hotkeys.none")}</span>}
            </button>
          </div>
        </div>
      ))}
      {msg && <div className="banner warn" role="alert"><span className="b-body">{msg}</span></div>}
      <div className="row-gap between" style={{ paddingTop: 14 }}>
        <span className="faint" style={{ fontSize: 12.5 }}>{t("hotkeys.hint")}</span>
        <button className="btn" onClick={() => set((d) => { d.hotkeys = { ...DEFAULT_HOTKEYS }; })}><RotateCcw size={15} />{t("hotkeys.reset")}</button>
      </div>
    </Card>
  );
}

export function StorageSection() {
  const t = useT();
  const { s, set, patch } = useS();
  const [info, setInfo] = useState<StorageInfo | null>(null);
  useEffect(() => { void ipc.storageInfo().then(setInfo).catch(() => undefined); }, [s.storage.recordingsDir]);
  return (
    <Card>
      <Field label={t("storage.folder")} hint={info && !info.exists ? t("storage.missing") : info ? t("storage.freeOf", { free: formatBytes(info.freeBytes), total: formatBytes(info.totalBytes, 0) }) : undefined} wide>
        <input className="input mono" dir="ltr" readOnly aria-label={t("storage.folder")} value={s.storage.recordingsDir} />
        <button className="btn" onClick={() => void pickFolder(s.storage.recordingsDir).then((p) => p && set((d) => { d.storage.recordingsDir = p; }))}><FolderOpen size={15} />{t("common.browse")}</button>
      </Field>
      <Field label={t("storage.library2")}><span className="dim num">{info ? t("storage.library", { size: formatBytes(info.libraryBytes), count: info.clipCount }) : "…"}</span></Field>
      <Field label={t("storage.lowWarn")} hint={t("storage.lowWarnHint")}><NumberInput label={t("storage.lowWarn")} min={1} max={500} value={s.storage.lowSpaceWarnGb} onChange={(v) => patch((d) => { d.storage.lowSpaceWarnGb = v; })} /><span className="dim">GB</span></Field>
      <Field label={t("storage.autoDelete")} hint={t("storage.autoDeleteHint")}><Toggle label={t("storage.autoDelete")} checked={s.storage.autoDelete} onChange={(v) => set((d) => { d.storage.autoDelete = v; })} /></Field>
      {s.storage.autoDelete && <Field label={t("storage.maxLibrary")} hint={t("storage.maxLibraryHint")}><NumberInput label={t("storage.maxLibrary")} min={1} max={20000} value={s.storage.maxLibraryGb || 500} onChange={(v) => patch((d) => { d.storage.maxLibraryGb = v; })} /><span className="dim">GB</span></Field>}
      <button className="btn" onClick={() => void ipc.openFolder(s.storage.recordingsDir).catch(() => undefined)}><FolderOpen size={15} />{t("storage.openFolder")}</button>
    </Card>
  );
}

function CornerPicker({ value, onChange, label }: { value: Corner; onChange: (c: Corner) => void; label: string }) {
  return (
    <div className="corner-picker" role="radiogroup" aria-label={label}>
      {(["top-left", "top-right", "bottom-left", "bottom-right"] as const).map((k) => <button key={k} role="radio" aria-checked={value === k} aria-label={k} className={`corner ${k}`} onClick={() => onChange(k)} />)}
    </div>
  );
}

export function PerformanceSection() {
  const t = useT();
  const { s, set } = useS();
  const p = s.performance;
  const toggleMetric = (m: string) => set((d) => { d.performance.statsMetrics = d.performance.statsMetrics.includes(m) ? d.performance.statsMetrics.filter((x) => x !== m) : [...d.performance.statsMetrics, m]; });
  return (
    <Card>
      <Field label={t("perf.stats")} hint={t("perf.statsHint")}><Toggle label={t("perf.stats")} checked={p.statsEnabled} onChange={(v) => set((d) => { d.performance.statsEnabled = v; })} /></Field>
      <div className="field" style={{ alignItems: "flex-start" }}>
        <div className="field-text"><div className="field-label">{t("perf.metrics")}</div><div className="field-hint">{t("perf.metricsHint")}</div></div>
        <div className="module-grid">{STAT_METRICS.map((m) => <label key={m} className="check"><input type="checkbox" checked={p.statsMetrics.includes(m)} onChange={() => toggleMetric(m)} /><span>{t(`perf.m.${m}` as "perf.m.fps")}</span></label>)}</div>
      </div>
      <Field label={t("perf.position")}><CornerPicker label={t("perf.position")} value={p.statsPosition} onChange={(c) => set((d) => { d.performance.statsPosition = c; })} /></Field>
      <Field label={t("perf.hud")} hint={t("perf.hudHint")}><Toggle label={t("perf.hud")} checked={p.hudEnabled} onChange={(v) => set((d) => { d.performance.hudEnabled = v; })} /></Field>
      <Field label={t("perf.hudPosition")}><CornerPicker label={t("perf.hudPosition")} value={p.hudPosition} onChange={(c) => set((d) => { d.performance.hudPosition = c; })} /></Field>
      <Field label={t("perf.hudInCapture")} hint={t("perf.hudInCaptureHint")}><Toggle label={t("perf.hudInCapture")} checked={p.hudIncludeInCapture} onChange={(v) => set((d) => { d.performance.hudIncludeInCapture = v; })} /></Field>
      <Field label={t("perf.notifications")} hint={t("perf.notificationsHint")}><Toggle label={t("perf.notifications")} checked={p.notifications} onChange={(v) => set((d) => { d.performance.notifications = v; })} /></Field>
    </Card>
  );
}

export function PrivacySection() {
  const t = useT();
  const { s, set } = useS();
  const wins = useDevices((d) => d.windows);
  const loadWindows = useDevices((d) => d.loadWindows);
  const [adding, setAdding] = useState(false);
  const [manual, setManual] = useState("");
  const apps = s.privacy.protectedApps;
  const add = (exe: string) => { const e = exe.trim(); if (!e) return; set((d) => { if (!d.privacy.protectedApps.some((a) => a.toLowerCase() === e.toLowerCase())) d.privacy.protectedApps.push(e); }); };
  const running = Array.from(new Map(wins.map((w) => [w.exe.toLowerCase(), w])).values());
  return (
    <Card>
      <InfoBanner>{t("privacy.intro")}</InfoBanner>
      <Field label={t("privacy.action")} hint={t("privacy.actionHint")}>
        <Segmented<"pause" | "blackout"> label={t("privacy.action")} value={s.privacy.action} onChange={(v) => set((d) => { d.privacy.action = v; })} options={[{ value: "blackout", label: t("privacy.blackout") }, { value: "pause", label: t("privacy.pause") }]} />
      </Field>
      <Field label={t("privacy.hideNotifications")} hint={t("privacy.hideNotificationsHint")}><Toggle label={t("privacy.hideNotifications")} checked={s.privacy.hideNotifications} onChange={(v) => set((d) => { d.privacy.hideNotifications = v; })} /></Field>
      <div className="field" style={{ alignItems: "flex-start" }}>
        <div className="field-text"><div className="field-label">{t("privacy.apps")}</div><div className="field-hint">{t("privacy.appsHint")}</div></div>
        <div style={{ flex: 1, maxWidth: 460 }}>
          <div className="chips" style={{ marginBottom: 10 }}>
            {apps.length === 0 && <span className="faint">{t("privacy.none")}</span>}
            {apps.map((a) => <span key={a} className="chip">{a}<button aria-label={t("common.delete")} onClick={() => set((d) => { d.privacy.protectedApps = d.privacy.protectedApps.filter((x) => x !== a); })}><X size={12} /></button></span>)}
          </div>
          <button className="btn sm" onClick={() => { void loadWindows(); setAdding(true); }}><Plus size={14} />{t("privacy.add")}</button>
        </div>
      </div>
      <Dialog open={adding} onClose={() => setAdding(false)} title={t("privacy.add")} actions={<button className="btn" onClick={() => setAdding(false)}>{t("common.done")}</button>}>
        <p className="dim" style={{ marginBottom: 10 }}>{t("privacy.pickRunning")}</p>
        <div className="pick-list">{running.map((w) => <button key={w.hwnd} className="opt-row" disabled={apps.some((a) => a.toLowerCase() === w.exe.toLowerCase())} onClick={() => add(w.exe)}><span>{w.title.slice(0, 60)}</span><span className="faint mono">{w.exe}</span></button>)}</div>
        <div className="row-gap" style={{ marginTop: 12 }}><input className="input mono" dir="ltr" placeholder="keepass.exe" aria-label={t("privacy.manual")} value={manual} onChange={(e) => setManual(e.target.value)} onKeyDown={(e) => { if (e.key === "Enter") { add(manual); setManual(""); } }} /><button className="btn" onClick={() => { add(manual); setManual(""); }}>{t("common.add")}</button></div>
      </Dialog>
    </Card>
  );
}

const ACCENTS = ["#7aa2ff", "#a78bfa", "#f472b6", "#fb923c", "#34d399", "#22d3ee", "#facc15"];

export function Appearance() {
  const t = useT();
  const { s, set, patch } = useS();
  const themes: { id: Settings["appearance"]["theme"]; bg: string; fg: string }[] = [
    { id: "midnight", bg: "#090b11", fg: "#161a24" }, { id: "graphite", bg: "#111215", fg: "#1f2228" }, { id: "oled", bg: "#000000", fg: "#0e1015" }, { id: "glass", bg: "linear-gradient(135deg,#1a1440,#0b0d14)", fg: "rgba(255,255,255,0.09)" },
  ];
  return (
    <Card>
      <Field label={t("appearance.theme")} wide>
        <div className="theme-picker" role="radiogroup" aria-label={t("appearance.theme")}>
          {themes.map((th) => (
            <button key={th.id} role="radio" aria-checked={s.appearance.theme === th.id} className="theme-card" onClick={() => set((d) => { d.appearance.theme = th.id; })}>
              <span className="theme-swatch" style={{ background: th.bg }}><i style={{ background: th.fg }} /><i style={{ background: "var(--accent)" }} /></span>
              <span>{t(`appearance.${th.id}` as "appearance.midnight")}</span>
            </button>
          ))}
        </div>
      </Field>
      <Field label={t("appearance.accent")} wide>
        <div className="swatches" role="radiogroup" aria-label={t("appearance.accent")}>
          {ACCENTS.map((c) => <button key={c} role="radio" aria-checked={s.appearance.accent.toLowerCase() === c} aria-label={c} className="swatch" style={{ background: c }} onClick={() => set((d) => { d.appearance.accent = c; })} />)}
          <input type="color" aria-label={t("appearance.customAccent")} className="swatch-input" value={s.appearance.accent} onChange={(e) => patch((d) => { d.appearance.accent = e.target.value; })} />
        </div>
      </Field>
      <Field label={t("appearance.uiScale")}><Segmented<number> label={t("appearance.uiScale")} value={s.appearance.uiScale} onChange={(v) => set((d) => { d.appearance.uiScale = v; })} options={[90, 100, 110, 125].map((n) => ({ value: n, label: `${n}%` }))} /></Field>
    </Card>
  );
}

export function LanguageSection() {
  const t = useT();
  return (
    <Card>
      <Field label={t("settings.language")} hint={t("language.hint")}><LanguageSwitch /></Field>
      <InfoBanner>{t("language.rtlNote")}</InfoBanner>
    </Card>
  );
}

export function Advanced() {
  const t = useT();
  const { s, set } = useS();
  const hw = useDevices((d) => d.hardware);
  const [info, setInfo] = useState<AppInfo | null>(null);
  const [probing, setProbing] = useState(false);
  useEffect(() => { void ipc.appInfo().then(setInfo); }, []);
  const reprobe = async () => { setProbing(true); try { useDevices.setState({ hardware: await ipc.reprobeHardware() }); } finally { setProbing(false); } };
  return (
    <>
      <Card>
        <Field label={t("adv.logLevel")} hint={t("adv.logHint")}><Select label={t("adv.logLevel")} value={s.logLevel} onChange={(v) => set((d) => { d.logLevel = v; })} options={["error", "warn", "info", "debug"].map((l) => ({ value: l, label: l.toUpperCase() }))} /></Field>
        <Field label={t("adv.logs")} hint={info?.logsDir}><button className="btn" onClick={() => void ipc.openLogsDir()}><ScrollText size={15} />{t("adv.openLogs")}</button></Field>
        <Field label={t("adv.version")}><span className="mono">{info?.version ?? "…"}</span></Field>
        <Field label={t("adv.data")}><code className="path" style={{ maxWidth: 360 }}>{info?.dataDir}</code></Field>
        <Field label={t("adv.ffmpeg")}><Chip tone={info?.ffmpegOk ? "ok" : "danger"}>{info?.ffmpegOk ? t("common.ok") : t("adv.ffmpegMissing")}</Chip></Field>
        <Field label={t("adv.streaming")} hint={t("adv.streamingHint")}><Chip tone="warn">{t("common.comingSoon")}</Chip></Field>
      </Card>
      <h2 className="section-title">{t("adv.encoders")}</h2>
      <Card>
        <div className="row-gap between" style={{ marginBottom: 8 }}><span className="dim">{t("adv.encodersHint")}</span><button className="btn" disabled={probing} onClick={() => void reprobe()}><RotateCcw size={15} className={probing ? "spin" : ""} />{t("adv.reprobe")}</button></div>
        {hw?.encoders.map((e) => (
          <div key={e.id} className="field"><div className="field-text"><div className="field-label">{e.label}</div>{e.error && !e.available && <div className="field-hint">{e.error}</div>}</div><Chip tone={e.available ? "ok" : undefined}>{e.available ? t("adv.available") : t("adv.unavailable")}</Chip></div>
        ))}
      </Card>
    </>
  );
}

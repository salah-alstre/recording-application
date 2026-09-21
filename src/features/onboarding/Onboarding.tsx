import { useEffect, useState } from "react";
import { open } from "@tauri-apps/plugin-dialog";
import { ArrowRight, Check, Cpu, FolderOpen, Mic } from "lucide-react";
import { Logo } from "@/components/Logo";
import { Chip, Meter } from "@/components/feedback";
import { Select } from "@/components/controls";
import { KeyCaps } from "@/components/controls";
import { useLang, useT } from "@/i18n";
import { useSettings } from "@/stores/settings";
import { useDevices } from "@/stores/devices";
import { useLevels } from "@/hooks/hooks";
import { ipc } from "@/lib/ipc";
import type { Lang, Quality } from "@/lib/types";
import { formatBytes, resolutionLabel } from "@/lib/format";
import { estimateBytes, pickEncoder } from "@/lib/encoders";
import { HOTKEY_ACTIONS } from "@/features/overlay/modules";

const STEPS = ["welcome", "language", "gpu", "folder", "mic", "quality", "hotkeys"] as const;

/** Seven-step first-launch flow. Every step maps to a real, persisted setting. */
export function Onboarding({ onFinish }: { onFinish: () => void }) {
  const t = useT();
  const [i, setI] = useState(0);
  const step = STEPS[i];
  const s = useSettings((x) => x.settings)!;
  const patch = useSettings((x) => x.patch);
  const hw = useDevices((d) => d.hardware);
  const audio = useDevices((d) => d.audio);
  const loadAudio = useDevices((d) => d.loadAudio);
  const lang = useLang((x) => x.lang);
  const [rec, setRec] = useState<Quality | null>(null);
  const lv = useLevels(step === "mic" && s.audio.micEnabled);

  useEffect(() => { void loadAudio(); }, [loadAudio]);
  useEffect(() => { if (step === "quality" && hw) void ipc.recommendQuality().then(setRec); }, [step, hw]);

  const next = () => setI((n) => Math.min(STEPS.length - 1, n + 1));
  const finish = async () => {
    if (rec) {
      const profiles = await ipc.listProfiles();
      const p = profiles.find((x) => x.id === s.activeProfileId) ?? profiles[0];
      if (p) await ipc.saveProfile({ ...p, quality: rec });
    }
    patch((d) => { d.general.onboarded = true; }, { immediate: true });
    onFinish();
  };
  const setLang = (l: Lang) => { useLang.getState().setLang(l); patch((d) => { d.general.language = l; }, { immediate: true }); };
  const gpu = hw?.gpus[0];
  const enc = rec && hw ? pickEncoder(rec, hw.encoders) : null;

  return (
    <div className="onboarding">
      <div className="onb-glow" aria-hidden />
      <div className="onb-card">
        <div className="onb-dots" aria-label={`${i + 1} / ${STEPS.length}`}>{STEPS.map((k, n) => <i key={k} className={n <= i ? "on" : ""} />)}</div>
        <div key={step} className="onb-step page-enter">
            {step === "welcome" && (<><Logo size={84} /><h1>{t("onb.welcome")}</h1><p className="dim">{t("onb.welcomeBody")}</p></>)}
            {step === "language" && (<><h1>{t("onb.language")}</h1><p className="dim">{t("onb.languageBody")}</p>
              <div className="lang-cards" role="radiogroup" aria-label={t("settings.language")}>
                {([["en", "English", "Left to right"], ["ar", "العربية", "من اليمين إلى اليسار"]] as const).map(([k, name, sub]) => (
                  <button key={k} role="radio" aria-checked={lang === k} className="lang-card" onClick={() => setLang(k)}><span className="lang-name" lang={k}>{name}</span><span className="dim" lang={k}>{sub}</span>{lang === k && <Check size={18} className="lang-check" />}</button>
                ))}</div></>)}
            {step === "gpu" && (<><h1>{t("onb.gpu")}</h1><p className="dim">{t("onb.gpuBody")}</p>
              {!hw ? <div className="skeleton" style={{ height: 130, width: "100%" }} /> : (
                <div className="onb-panel"><Cpu size={20} className="accent-text" />
                  <div style={{ flex: 1 }}><div className="big-value">{gpu?.name ?? t("encoder.noGpu")}</div><div className="dim" style={{ fontSize: 13 }}>{hw.cpu} · {formatBytes(hw.ramBytes, 0)}</div>
                    <div className="chips" style={{ marginTop: 10 }}>{hw.encoders.filter((e) => e.available).map((e) => <Chip key={e.id} tone={e.vendor === "cpu" ? undefined : "ok"}>{e.label}</Chip>)}</div></div></div>)}</>)}
            {step === "folder" && (<><h1>{t("onb.folder")}</h1><p className="dim">{t("onb.folderBody")}</p>
              <div className="onb-panel"><FolderOpen size={20} className="accent-text" /><code className="path" style={{ flex: 1 }}>{s.storage.recordingsDir}</code>
                <button className="btn" onClick={() => void open({ directory: true, defaultPath: s.storage.recordingsDir }).then((p) => typeof p === "string" && patch((d) => { d.storage.recordingsDir = p; }, { immediate: true }))}>{t("common.browse")}</button></div></>)}
            {step === "mic" && (<><h1>{t("onb.mic")}</h1><p className="dim">{t("onb.micBody")}</p>
              <div className="onb-panel col"><div className="row-gap" style={{ width: "100%" }}><Mic size={20} className="accent-text" />
                <div style={{ flex: 1 }}><Select label={t("mic.device")} value={s.audio.micDevice} options={[{ value: "", label: t("audio.defaultDevice") }, ...audio.inputs.map((d) => ({ value: d.name, label: d.name }))]} onChange={(v) => patch((d) => { d.audio.micDevice = v; d.audio.micEnabled = true; }, { immediate: true })} /></div></div>
                <Meter level={lv.mic} off={!s.audio.micEnabled} label={t("mic.level")} />
                <label className="row-gap"><input type="checkbox" checked={s.audio.micEnabled} onChange={(e) => patch((d) => { d.audio.micEnabled = e.target.checked; }, { immediate: true })} /><span>{t("onb.micEnable")}</span></label></div></>)}
            {step === "quality" && (<><h1>{t("onb.quality")}</h1><p className="dim">{t("onb.qualityBody")}</p>
              {!rec ? <div className="skeleton" style={{ height: 130, width: "100%" }} /> : (
                <div className="onb-panel col"><div className="chips"><Chip tone="accent">{resolutionLabel(rec.height)}</Chip><Chip>{rec.fps} fps</Chip><Chip>{rec.codec.toUpperCase()}</Chip><Chip>{enc?.label ?? t("encoder.none")}</Chip></div>
                  <div className="dim num">{t("quality.estimate", { size: formatBytes(estimateBytes(rec, 3, 10), 0), min: 10 })}</div>
                  <div className="faint" style={{ fontSize: 12.5 }}>{t("onb.qualityOverride")}</div></div>)}</>)}
            {step === "hotkeys" && (<><h1>{t("onb.hotkeys")}</h1><p className="dim">{t("onb.hotkeysBody")}</p>
              <div className="onb-panel col">{HOTKEY_ACTIONS.slice(0, 6).map((a) => <div key={a} className="row-gap between" style={{ width: "100%" }}><span>{t(`hotkeys.action.${a}` as "hotkeys.action.record")}</span><KeyCaps accel={s.hotkeys[a] ?? ""} /></div>)}</div></>)}
        </div>
        <div className="onb-actions">
          {i > 0 ? <button className="btn ghost" onClick={() => setI(i - 1)}>{t("common.back")}</button> : <span />}
          {i < STEPS.length - 1 ? <button className="btn primary" onClick={next}>{t("common.continue")}<ArrowRight size={16} className="rtl-flip" /></button>
            : <button className="btn primary" onClick={() => void finish()}><Check size={16} />{t("onb.finish")}</button>}
        </div>
      </div>
    </div>
  );
}

import { useEffect, useMemo, useState } from "react";
import { Cpu, FolderOpen, HardDrive, Layers, Mic, MicOff, RotateCcw, Volume2, VolumeX, Zap } from "lucide-react";
import { Chip, Meter } from "@/components/feedback";
import { Select, Toggle, type Option } from "@/components/controls";
import { useT } from "@/i18n";
import { useSettings } from "@/stores/settings";
import { useDevices } from "@/stores/devices";
import { useRecording } from "@/stores/recording";
import { useUi } from "@/stores/ui";
import { useLevels, useVisible } from "@/hooks/hooks";
import { actions } from "@/features/common/actions";
import { ipc } from "@/lib/ipc";
import type { Profile, StorageInfo } from "@/lib/types";
import { formatBytes, percent, resolutionLabel } from "@/lib/format";
import { estimateBytes, pickEncoder } from "@/lib/encoders";
import { ClipThumb } from "@/features/library/ClipCard";
import { useLibrary } from "@/stores/library";

export const REPLAY_DURATIONS = [15, 30, 60, 120, 180, 300, 600, 1200];

export function durationLabel(t: ReturnType<typeof useT>, secs: number): string {
  return secs < 60 ? t("unit.seconds", { n: secs }) : t("unit.minutes", { n: secs / 60 });
}

export function ReplayCard() {
  const t = useT();
  const s = useSettings((x) => x.settings);
  const patch = useSettings((x) => x.patch);
  const replay = useRecording((x) => x.status.replay);
  const profile = useProfile();
  if (!s) return null;
  const bufferMb = profile ? Math.round((profile.quality.bitrateKbps * s.replay.durationSecs) / 8 / 1000) : 0;
  const opts: Option<number>[] = REPLAY_DURATIONS.map((d) => ({ value: d, label: durationLabel(t, d) }));
  if (!REPLAY_DURATIONS.includes(s.replay.durationSecs)) opts.push({ value: s.replay.durationSecs, label: durationLabel(t, s.replay.durationSecs) });
  return (
    <section className="card" aria-labelledby="replay-title">
      <div className="card-title" id="replay-title"><RotateCcw size={15} />{t("replay.title")}<span style={{ flex: 1 }} />
        <Toggle label={t("replay.title")} checked={!!replay} onChange={() => void actions.toggleReplay()} /></div>
      <div className="stack">
        <div className="row-gap"><span className="dim" style={{ flex: 1 }}>{t("replay.length")}</span>
          <div style={{ width: 150 }}><Select label={t("replay.length")} value={s.replay.durationSecs} options={opts} onChange={(v) => patch((d) => { d.replay.durationSecs = v; }, { immediate: true })} /></div></div>
        <div className="dim" style={{ fontSize: 12.5 }}>{t("replay.bufferNote", { mb: bufferMb })}</div>
        <button className="btn primary" disabled={!replay || replay.saving} onClick={() => void actions.saveReplay()}>
          <RotateCcw size={16} />{replay?.saving ? t("replay.saving") : t("replay.saveLast", { len: durationLabel(t, s.replay.durationSecs) })}
        </button>
      </div>
    </section>
  );
}

export function useProfile(): Profile | undefined {
  const profiles = useDevices((s) => s.profiles);
  const id = useSettings((s) => s.settings?.activeProfileId);
  return profiles.find((p) => p.id === id) ?? profiles[0];
}

export function AudioCard() {
  const t = useT();
  const s = useSettings((x) => x.settings);
  const patch = useSettings((x) => x.patch);
  const visible = useVisible();
  const lv = useLevels(visible);
  if (!s) return null;
  const a = s.audio;
  return (
    <section className="card" aria-labelledby="audio-title">
      <div className="card-title" id="audio-title"><Volume2 size={15} />{t("audio.title")}</div>
      <div className="stack">
        <div className="mixer-row">
          <button className="btn icon sm ghost" aria-label={t("audio.system")} onClick={() => patch((d) => { d.audio.systemEnabled = !d.audio.systemEnabled; }, { immediate: true })}>{a.systemEnabled ? <Volume2 size={17} /> : <VolumeX size={17} className="faint" />}</button>
          <div className="mixer-body"><div className="mixer-label"><span>{t("audio.system")}</span><span className="mono dim">{a.systemVolume}%</span></div>
            <Meter level={lv.system} off={!a.systemEnabled} label={t("audio.system")} />
            <input type="range" className="slider" aria-label={t("audio.systemVolume")} min={0} max={200} value={a.systemVolume} style={{ ["--p" as string]: `${a.systemVolume / 2}%` }} onChange={(e) => patch((d) => { d.audio.systemVolume = Number(e.target.value); })} /></div></div>
        <div className="mixer-row">
          <button className="btn icon sm ghost" aria-label={t("audio.mic")} onClick={() => actions.toggleMic()}>{a.micEnabled ? <Mic size={17} /> : <MicOff size={17} className="faint" />}</button>
          <div className="mixer-body"><div className="mixer-label"><span>{t("audio.mic")}</span><span className="mono dim">{a.micVolume}%</span></div>
            <Meter level={lv.mic} off={!a.micEnabled} label={t("audio.mic")} />
            <input type="range" className="slider" aria-label={t("audio.micVolume")} min={0} max={200} value={a.micVolume} style={{ ["--p" as string]: `${a.micVolume / 2}%` }} onChange={(e) => patch((d) => { d.audio.micVolume = Number(e.target.value); })} /></div></div>
      </div>
    </section>
  );
}

export function EncoderCard() {
  const t = useT();
  const hw = useDevices((s) => s.hardware);
  const profile = useProfile();
  const enc = useMemo(() => (hw && profile ? pickEncoder(profile.quality, hw.encoders) : null), [hw, profile]);
  const gpu = hw?.gpus[0];
  return (
    <section className="card" aria-labelledby="enc-title">
      <div className="card-title" id="enc-title"><Zap size={15} />{t("encoder.title")}</div>
      {!hw ? (
        <div className="stack"><div className="skeleton" style={{ height: 22, width: "70%" }} /><div className="skeleton" style={{ height: 16, width: "90%" }} /><div className="skeleton" style={{ height: 16, width: "60%" }} /></div>
      ) : (
        <div className="stack">
          <div className="big-value">{enc?.label ?? t("encoder.none")}</div>
          <div className="dim"><Cpu size={13} style={{ verticalAlign: -2 }} /> {gpu ? `${gpu.name}${gpu.vramBytes ? ` · ${formatBytes(gpu.vramBytes, 0)}` : ""}` : t("encoder.noGpu")}</div>
          <div className="dim" style={{ fontSize: 12.5 }}>{hw.cpu} · {formatBytes(hw.ramBytes, 0)}</div>
          <div className="chips">{hw.encoders.filter((e) => e.available && e.vendor !== "cpu").map((e) => <Chip key={e.id} tone={e.id === enc?.id ? "accent" : undefined}>{e.label.replace(/^(NVIDIA|AMD|Intel) /, "")}</Chip>)}</div>
        </div>
      )}
    </section>
  );
}

export function StorageCard() {
  const t = useT();
  const [info, setInfo] = useState<StorageInfo | null>(null);
  const dir = useSettings((s) => s.settings?.storage.recordingsDir);
  useEffect(() => { void ipc.storageInfo().then(setInfo).catch(() => undefined); }, [dir]);
  const used = info ? 100 - percent(info.freeBytes, info.totalBytes) : 0;
  return (
    <section className="card" aria-labelledby="stor-title">
      <div className="card-title" id="stor-title"><HardDrive size={15} />{t("storage.title")}</div>
      {!info ? <div className="skeleton" style={{ height: 60 }} /> : (
        <div className="stack">
          {!info.exists && <div className="chip warn">{t("storage.missing")}</div>}
          <div className="big-value num">{formatBytes(info.freeBytes)} <span className="dim" style={{ fontSize: 13, fontWeight: 500 }}>{t("storage.free")}</span></div>
          <div className="progress"><i style={{ width: `${used}%`, background: used > 90 ? "var(--danger)" : undefined }} /></div>
          <div className="dim num" style={{ fontSize: 12.5 }}>{t("storage.library", { size: formatBytes(info.libraryBytes), count: info.clipCount })}</div>
          <div className="row-gap"><code className="path" title={info.dir}>{info.dir}</code>
            <button className="btn icon sm" aria-label={t("storage.openFolder")} onClick={() => void ipc.openFolder(info.dir).catch(() => undefined)}><FolderOpen size={15} /></button></div>
        </div>
      )}
    </section>
  );
}

export function ProfileCard() {
  const t = useT();
  const profiles = useDevices((s) => s.profiles);
  const profile = useProfile();
  const go = useUi((s) => s.go);
  const replace = useSettings((s) => s.replace);
  const tracks = 3;
  if (!profile) return <section className="card"><div className="skeleton" style={{ height: 90 }} /></section>;
  const q = profile.quality;
  return (
    <section className="card" aria-labelledby="prof-title">
      <div className="card-title" id="prof-title"><Layers size={15} />{t("profiles.active")}<span style={{ flex: 1 }} /><button className="btn ghost sm" onClick={() => go("profiles")}>{t("common.manage")}</button></div>
      <div className="stack">
        <Select label={t("profiles.active")} value={profile.id} options={profiles.map((p) => ({ value: p.id, label: p.name }))}
          onChange={(id) => void ipc.setActiveProfile(id).then(replace)} />
        <div className="chips"><Chip tone="accent">{resolutionLabel(q.height)}</Chip><Chip>{q.fps} fps</Chip><Chip>{q.codec.toUpperCase()}</Chip><Chip>{(q.bitrateKbps / 1000).toFixed(0)} Mbps</Chip></div>
        <div className="dim num" style={{ fontSize: 12.5 }}>{t("quality.estimate", { size: formatBytes(estimateBytes(q, tracks, 10), 0), min: 10 })}</div>
      </div>
    </section>
  );
}

export function RecentClips() {
  const t = useT();
  const clips = useLibrary((s) => s.clips);
  const loaded = useLibrary((s) => s.loaded);
  const go = useUi((s) => s.go);
  const play = useUi((s) => s.play);
  const recent = clips.slice(0, 6);
  return (
    <section className="card recent" aria-labelledby="recent-title">
      <div className="card-title" id="recent-title">{t("home.recent")}<span style={{ flex: 1 }} /><button className="btn ghost sm" onClick={() => go("library")}>{t("home.viewAll")}</button></div>
      {!loaded ? (
        <div className="recent-row">{Array.from({ length: 4 }).map((_, i) => <div key={i} className="skeleton" style={{ aspectRatio: "16/9" }} />)}</div>
      ) : recent.length === 0 ? (
        <p className="dim" style={{ padding: "18px 4px" }}>{t("home.noClips")}</p>
      ) : (
        <div className="recent-row">{recent.map((c) => (
          <button key={c.id} className="recent-item" onClick={() => { go("library"); play(c.id); }} aria-label={c.title}><ClipThumb clip={c} /></button>
        ))}</div>
      )}
    </section>
  );
}

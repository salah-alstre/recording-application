import { useEffect, useState } from "react";
import { Check, Copy, Gamepad2, Layers, Pencil, Plus, Trash2, X } from "lucide-react";
import { useT } from "@/i18n";
import { useDevices } from "@/stores/devices";
import { useSettings } from "@/stores/settings";
import { useRecording } from "@/stores/recording";
import { ipc } from "@/lib/ipc";
import { attempt } from "@/features/common/actions";
import type { GameConfig, Profile } from "@/lib/types";
import { Chip } from "@/components/feedback";
import { Dialog } from "@/components/overlays";
import { Field, Select, Toggle } from "@/components/controls";
import { QualityEditor } from "@/features/profiles/QualityEditor";
import { resolutionLabel } from "@/lib/format";
import type { Quality } from "@/lib/types";

const blankQuality = (): Quality => ({ preset: "high", height: 1080, fps: 60, bitrateKbps: 20000, codec: "h264", encoder: "auto", rateControl: "cbr", cq: 22, speed: "balanced", container: "mp4", audioBitrateKbps: 160 });

export function Profiles() {
  const t = useT();
  const profiles = useDevices((s) => s.profiles);
  const load = useDevices((s) => s.loadProfiles);
  const activeId = useSettings((s) => s.settings?.activeProfileId);
  const replace = useSettings((s) => s.replace);
  const game = useRecording((s) => s.status.game);
  const [editing, setEditing] = useState<Profile | null>(null);
  const [apps, setApps] = useState("");
  const openEditor = (p: Profile) => { setApps(p.autoApps.join(", ")); setEditing(p); };
  const [games, setGames] = useState<GameConfig[]>([]);
  const loadGames = () => void ipc.listGameConfigs().then(setGames).catch(() => undefined);
  useEffect(() => { void load(); loadGames(); }, [load]);

  const fresh = (): Profile => ({ id: "", name: "", quality: blankQuality(), micEnabled: false, systemEnabled: true, cameraEnabled: false, autoApps: [], builtin: false });
  const save = async () => {
    if (!editing || !editing.name.trim()) return;
    const p = { ...editing, autoApps: Array.from(new Set(apps.split(/[,\n;]/).map((a) => a.trim()).filter(Boolean))) };
    await attempt(ipc.saveProfile(p));
    setEditing(null); setApps("");
    await load();
    if (p.id === activeId) void ipc.setActiveProfile(p.id).then(replace);
  };

  return (
    <div className="page">
      <div className="page-head"><h1 className="page-title">{t("nav.profiles")}</h1>
        <button className="btn primary" onClick={() => openEditor(fresh())}><Plus size={16} />{t("profiles.new")}</button></div>
      <p className="dim" style={{ marginBottom: 18, maxWidth: 640 }}>{t("profiles.intro")}</p>

      <div className="profile-grid">
        {profiles.map((p) => (
          <article key={p.id} className={`card profile${p.id === activeId ? " active" : ""}`}>
            <div className="profile-head"><Layers size={17} className="accent-text" /><h3>{p.name}</h3>{p.id === activeId && <Chip tone="accent" icon={<Check size={12} />}>{t("profiles.activeBadge")}</Chip>}</div>
            <div className="chips"><Chip>{resolutionLabel(p.quality.height)}</Chip><Chip>{p.quality.fps} fps</Chip><Chip>{p.quality.codec.toUpperCase()}</Chip><Chip>{(p.quality.bitrateKbps / 1000).toFixed(0)} Mbps</Chip><Chip>{p.quality.container.toUpperCase()}</Chip></div>
            <div className="dim" style={{ fontSize: 12.5 }}>{[p.systemEnabled && t("audio.system"), p.micEnabled && t("audio.mic"), p.cameraEnabled && t("camera.title")].filter(Boolean).join(" · ") || t("profiles.noAudio")}</div>
            {p.autoApps.length > 0 && <div className="chips">{p.autoApps.map((a) => <Chip key={a} icon={<Gamepad2 size={12} />}>{a}</Chip>)}</div>}
            <div className="profile-actions">
              {p.id !== activeId && <button className="btn sm primary" onClick={() => void ipc.setActiveProfile(p.id).then(replace)}>{t("profiles.use")}</button>}
              <button className="btn sm" onClick={() => openEditor(p)}><Pencil size={14} />{t("common.edit")}</button>
              <button className="btn sm" aria-label={t("profiles.duplicate")} onClick={() => void attempt(ipc.saveProfile({ ...p, id: "", name: `${p.name} ${t("profiles.copySuffix")}`, builtin: false })).then(load)}><Copy size={14} /></button>
              {profiles.length > 1 && <button className="btn sm danger" aria-label={t("common.delete")} onClick={() => void attempt(ipc.deleteProfile(p.id)).then(load)}><Trash2 size={14} /></button>}
            </div>
          </article>
        ))}
      </div>

      <h2 className="section-title">{t("profiles.games")}</h2>
      <p className="dim" style={{ marginBottom: 12 }}>{t("profiles.gamesIntro")}</p>
      {game && !games.some((g) => g.exe.toLowerCase() === game.exe.toLowerCase()) && (
        <div className="banner" style={{ marginBottom: 12 }}>
          <Gamepad2 size={18} className="accent-text" />
          <div style={{ flex: 1 }}><div className="b-title">{t("profiles.detected", { name: game.name })}</div><div className="b-body">{game.exe}</div></div>
          <button className="btn sm primary" onClick={() => void ipc.saveGameConfig({ exe: game.exe, name: game.name, profileId: activeId ?? profiles[0]?.id ?? "", camera: null }).then(loadGames)}>{t("profiles.remember")}</button>
        </div>
      )}
      {games.length === 0 ? <p className="faint">{t("profiles.noGames")}</p> : (
        <div className="card" style={{ padding: "6px 20px" }}>
          {games.map((g) => (
            <div key={g.exe} className="field">
              <div className="field-text"><div className="field-label">{g.name}</div><div className="field-hint">{g.exe}</div></div>
              <div className="field-control">
                <Select label={g.name} value={g.profileId} options={profiles.map((p) => ({ value: p.id, label: p.name }))} onChange={(v) => void ipc.saveGameConfig({ ...g, profileId: v }).then(loadGames)} />
                <button className="btn icon sm danger" aria-label={t("common.delete")} onClick={() => void ipc.deleteGameConfig(g.exe).then(loadGames)}><X size={14} /></button>
              </div>
            </div>
          ))}
        </div>
      )}

      <Dialog wide open={!!editing} onClose={() => setEditing(null)} title={editing?.id ? t("profiles.edit") : t("profiles.new")}
        actions={<><button className="btn" onClick={() => setEditing(null)}>{t("common.cancel")}</button><button className="btn primary" disabled={!editing?.name.trim()} onClick={() => void save()}>{t("common.save")}</button></>}>
        {editing && (
          <div>
            <Field label={t("profiles.name")}><input className="input" autoFocus aria-label={t("profiles.name")} value={editing.name} onChange={(e) => setEditing({ ...editing, name: e.target.value })} /></Field>
            <div className="field"><div className="field-text"><div className="field-label">{t("profiles.audioSources")}</div></div>
              <div className="field-control wide" style={{ gap: 18 }}>
                <label className="row-gap"><Toggle label={t("audio.system")} checked={editing.systemEnabled} onChange={(v) => setEditing({ ...editing, systemEnabled: v })} />{t("audio.system")}</label>
                <label className="row-gap"><Toggle label={t("audio.mic")} checked={editing.micEnabled} onChange={(v) => setEditing({ ...editing, micEnabled: v })} />{t("audio.mic")}</label>
                <label className="row-gap"><Toggle label={t("camera.title")} checked={editing.cameraEnabled} onChange={(v) => setEditing({ ...editing, cameraEnabled: v })} />{t("camera.title")}</label>
              </div></div>
            <Field label={t("profiles.autoApps")} hint={t("profiles.autoAppsHint")} wide>
              <input className="input mono" aria-label={t("profiles.autoApps")} placeholder="game.exe, obs.exe" value={apps} onChange={(e) => setApps(e.target.value)} />
            </Field>
            <QualityEditor q={editing.quality} onChange={(q) => setEditing({ ...editing, quality: q })} />
          </div>
        )}
      </Dialog>
    </div>
  );
}

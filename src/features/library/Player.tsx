import { useCallback, useEffect, useRef, useState } from "react";
import { ArrowLeft, ExternalLink, FolderOpen, Maximize, Pause, Pencil, Play, Scissors, Star, Trash2, Volume2, VolumeX, Flag } from "lucide-react";
import { useT, useLang } from "@/i18n";
import { ipc } from "@/lib/ipc";
import type { Clip, Marker } from "@/lib/types";
import { assetUrl } from "@/hooks/hooks";
import { formatBytes, formatDate, formatDuration, resolutionLabel } from "@/lib/format";
import { Chip, Skeleton } from "@/components/feedback";
import { Select } from "@/components/controls";
import { ErrorBanner, InfoBanner } from "@/components/overlays";
import { Editor } from "./Editor";
import { attempt } from "@/features/common/actions";
import { useLibrary } from "@/stores/library";

const SPEEDS = [0.5, 0.75, 1, 1.25, 1.5, 2];

export function Player({ id, onClose }: { id: string; onClose: () => void }) {
  const t = useT();
  const lang = useLang((s) => s.lang);
  const [clip, setClip] = useState<Clip | null>(null);
  const [markers, setMarkers] = useState<Marker[]>([]);
  const [editing, setEditing] = useState(false);
  const [notes, setNotes] = useState("");
  const [playError, setPlayError] = useState(false);
  const load = useCallback(async () => {
    const c = await ipc.getClip(id);
    setClip(c);
    setNotes(c?.notes ?? "");
    setMarkers(await ipc.listMarkers(id).catch(() => []));
  }, [id]);
  useEffect(() => { void load(); }, [load]);

  if (!clip) return <div className="page"><Skeleton h={420} r={22} /></div>;
  const isImage = clip.kind === "screenshot";
  const src = assetUrl(clip.path);

  return (
    <div className="page player-page">
      <div className="page-head">
        <button className="btn" onClick={onClose}><ArrowLeft size={16} className="rtl-flip" />{t("common.back")}</button>
        <h1 className="page-title player-title" title={clip.title}>{clip.title}</h1>
        <div className="row-gap">
          <button className="btn icon" aria-pressed={clip.favorite} aria-label={t("library.favorite")} onClick={() => void ipc.setFavorite(clip.id, !clip.favorite).then(load).then(() => useLibrary.getState().refresh())}><Star size={16} fill={clip.favorite ? "currentColor" : "none"} style={{ color: clip.favorite ? "var(--warn)" : undefined }} /></button>
          {!isImage && <button className="btn" onClick={() => setEditing((v) => !v)} aria-pressed={editing}><Scissors size={16} />{t("editor.title")}</button>}
          <button className="btn icon" aria-label={t("library.reveal")} onClick={() => void ipc.revealClip(clip.id).catch(() => undefined)}><FolderOpen size={16} /></button>
          <button className="btn icon danger" aria-label={t("library.delete")} onClick={() => void attempt(ipc.deleteClip(clip.id)).then(() => { void useLibrary.getState().refresh(); onClose(); })}><Trash2 size={16} /></button>
        </div>
      </div>

      <div className="player-layout">
        <div className="player-main">
          {isImage ? (
            <div className="viewer"><img src={src} alt={clip.title} /></div>
          ) : editing ? (
            <Editor clip={clip} src={src} onDone={(path) => { setEditing(false); void useLibrary.getState().refresh(); void ipc.scanLibrary().then(() => useLibrary.getState().refresh()); void path; }} />
          ) : (
            <VideoView clip={clip} src={src} markers={markers} onError={() => setPlayError(true)} />
          )}
          {playError && !isImage && (
            <div style={{ marginTop: 14 }}>
              <ErrorBanner error={{ code: "codec_unsupported", message: "This video uses a codec the built-in player cannot decode.", hint: "Open it with another player." }}
                action={<button className="btn sm" onClick={() => void ipc.openPath(clip.path).catch(() => undefined)}><ExternalLink size={14} />{t("player.openExternal")}</button>} />
            </div>
          )}
        </div>

        <aside className="player-side">
          <div className="card">
            <div className="card-title">{t("player.details")}</div>
            <div className="chips" style={{ marginBottom: 12 }}>
              <Chip tone="accent">{isImage ? `${clip.width}×${clip.height}` : `${clip.width}×${clip.height} · ${resolutionLabel(clip.height)}`}</Chip>
              {!isImage && <Chip>{Math.round(clip.fps)} fps</Chip>}
              {clip.codec && <Chip>{clip.codec.toUpperCase()}</Chip>}
              {clip.encoder && <Chip>{clip.encoder.replace(/^(NVIDIA|AMD|Intel) /, "")}</Chip>}
              {!isImage && <Chip>{formatDuration(clip.durationMs)}</Chip>}
              <Chip>{formatBytes(clip.sizeBytes)}</Chip>
              {clip.audioTracks > 0 && <Chip>{t("player.tracks", { n: clip.audioTracks })}</Chip>}
            </div>
            <dl className="meta">
              <dt>{t("player.game")}</dt><dd>{clip.game || t("library.desktop")}</dd>
              <dt>{t("player.created")}</dt><dd>{formatDate(clip.createdAt, lang)}</dd>
              <dt>{t("player.path")}</dt><dd className="path-wrap" title={clip.path}>{clip.path}</dd>
            </dl>
          </div>
          {!isImage && (
            <div className="card">
              <div className="card-title"><Flag size={14} />{t("player.markers")}</div>
              {markers.length === 0 ? <p className="dim" style={{ fontSize: 13 }}>{t("player.noMarkers")}</p> : (
                <ul className="marker-list">{markers.map((m) => (
                  <li key={m.id}><button className="marker-item" onClick={() => { const v = document.querySelector("video"); if (v) { v.currentTime = m.tMs / 1000; void v.play(); } }}><Flag size={13} /><span className="mono num">{formatDuration(m.tMs)}</span><span>{m.label || t("player.marker")}</span></button>
                    <button className="btn ghost icon sm" aria-label={t("common.delete")} onClick={() => void ipc.deleteMarker(m.id).then(load)}><Trash2 size={13} /></button></li>
                ))}</ul>
              )}
            </div>
          )}
          <div className="card">
            <div className="card-title"><Pencil size={14} />{t("player.notes")}</div>
            <textarea className="input notes" rows={4} placeholder={t("player.notesPlaceholder")} aria-label={t("player.notes")} value={notes} onChange={(e) => setNotes(e.target.value)} onBlur={() => void ipc.setNotes(clip.id, notes)} />
          </div>
          {!isImage && <InfoBanner>{t("player.multitrackNote")}</InfoBanner>}
        </aside>
      </div>
    </div>
  );
}

function VideoView({ clip, src, markers, onError }: { clip: Clip; src: string; markers: Marker[]; onError: () => void }) {
  const t = useT();
  const v = useRef<HTMLVideoElement>(null);
  const box = useRef<HTMLDivElement>(null);
  const [playing, setPlaying] = useState(false);
  const [time, setTime] = useState(0);
  const [dur, setDur] = useState(clip.durationMs / 1000);
  const [vol, setVol] = useState(1);
  const [muted, setMuted] = useState(false);
  const [speed, setSpeed] = useState(1);

  const toggle = useCallback(() => { const el = v.current; if (!el) return; if (el.paused) void el.play(); else el.pause(); }, []);
  const seek = useCallback((s: number) => { const el = v.current; if (el) el.currentTime = Math.max(0, Math.min(el.duration || 0, s)); }, []);

  useEffect(() => {
    const k = (e: KeyboardEvent) => {
      if ((e.target as HTMLElement)?.tagName === "TEXTAREA" || (e.target as HTMLElement)?.tagName === "INPUT") return;
      if (e.key === " ") { e.preventDefault(); toggle(); }
      else if (e.key === "ArrowRight") seek((v.current?.currentTime ?? 0) + 5);
      else if (e.key === "ArrowLeft") seek((v.current?.currentTime ?? 0) - 5);
      else if (e.key.toLowerCase() === "f") void (document.fullscreenElement ? document.exitFullscreen() : box.current?.requestFullscreen());
      else if (e.key.toLowerCase() === "m") setMuted((m) => !m);
    };
    window.addEventListener("keydown", k);
    return () => window.removeEventListener("keydown", k);
  }, [toggle, seek]);

  useEffect(() => { if (v.current) { v.current.volume = vol; v.current.muted = muted; v.current.playbackRate = speed; } }, [vol, muted, speed]);

  return (
    <div className="video-box" ref={box}>
      <video ref={v} src={src} onClick={toggle} onDoubleClick={() => void box.current?.requestFullscreen()} onPlay={() => setPlaying(true)} onPause={() => setPlaying(false)}
        onTimeUpdate={(e) => setTime(e.currentTarget.currentTime)} onLoadedMetadata={(e) => Number.isFinite(e.currentTarget.duration) && setDur(e.currentTarget.duration)} onError={onError} preload="metadata" />
      <div className="controls">
        <button className="btn icon sm ghost" aria-label={playing ? t("player.pause") : t("player.play")} onClick={toggle}>{playing ? <Pause size={18} /> : <Play size={18} />}</button>
        <span className="mono num time">{formatDuration(time * 1000)}</span>
        <div className="seek">
          <input type="range" className="slider" aria-label={t("player.seek")} min={0} max={dur || 1} step={0.05} value={time} style={{ ["--p" as string]: `${(time / (dur || 1)) * 100}%` }} onChange={(e) => seek(Number(e.target.value))} />
          {markers.map((m) => <i key={m.id} className="seek-marker" style={{ insetInlineStart: `${(m.tMs / 1000 / (dur || 1)) * 100}%` }} title={formatDuration(m.tMs)} />)}
        </div>
        <span className="mono num time dim">{formatDuration(dur * 1000)}</span>
        <button className="btn icon sm ghost" aria-label={t("player.mute")} onClick={() => setMuted(!muted)}>{muted || vol === 0 ? <VolumeX size={17} /> : <Volume2 size={17} />}</button>
        <input type="range" className="slider vol" aria-label={t("player.volume")} min={0} max={1} step={0.02} value={muted ? 0 : vol} style={{ ["--p" as string]: `${(muted ? 0 : vol) * 100}%` }} onChange={(e) => { setVol(Number(e.target.value)); setMuted(false); }} />
        <div style={{ width: 84 }}><Select label={t("player.speed")} value={speed} options={SPEEDS.map((s) => ({ value: s, label: `${s}×` }))} onChange={setSpeed} /></div>
        <button className="btn icon sm ghost" aria-label={t("player.fullscreen")} onClick={() => void box.current?.requestFullscreen()}><Maximize size={17} /></button>
      </div>
    </div>
  );
}

export { InfoBanner };

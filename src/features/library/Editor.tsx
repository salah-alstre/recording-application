import { useEffect, useMemo, useRef, useState } from "react";
import { Check, RotateCw, Scissors, Type, Volume2, VolumeX } from "lucide-react";
import { Segmented, Slider, Toggle, Field } from "@/components/controls";
import { ErrorBanner } from "@/components/overlays";
import { Progress } from "@/components/feedback";
import { useT } from "@/i18n";
import { useTauriEvent } from "@/hooks/hooks";
import { ipc, normalizeError } from "@/lib/ipc";
import type { AppError, Clip, ExportOpts } from "@/lib/types";
import { formatDuration } from "@/lib/format";

type CropPreset = "none" | "16:9" | "1:1" | "9:16" | "4:3";

/** Centre crop for an aspect preset, in source pixels (even dimensions). */
export function cropFor(preset: CropPreset, w: number, h: number): { x: number; y: number; w: number; h: number } | null {
  if (preset === "none") return null;
  const [a, b] = preset.split(":").map(Number);
  const target = a / b;
  let cw = w, ch = Math.round(w / target);
  if (ch > h) { ch = h; cw = Math.round(h * target); }
  cw &= ~1; ch &= ~1;
  return { x: Math.round((w - cw) / 2), y: Math.round((h - ch) / 2), w: cw, h: ch };
}

export function Editor({ clip, src, onDone }: { clip: Clip; src: string; onDone: (path: string) => void }) {
  const t = useT();
  const v = useRef<HTMLVideoElement>(null);
  const total = clip.durationMs;
  const [range, setRange] = useState<[number, number]>([0, total]);
  const [crop, setCrop] = useState<CropPreset>("none");
  const [rotate, setRotate] = useState(0);
  const [mute, setMute] = useState(false);
  const [volume, setVolume] = useState(100);
  const [text, setText] = useState("");
  const [textPos, setTextPos] = useState<"top" | "center" | "bottom">("bottom");
  const [progress, setProgress] = useState<number | null>(null);
  const [error, setError] = useState<AppError | null>(null);
  const [done, setDone] = useState(false);

  useTauriEvent<{ id: string; pct: number }>("export-progress", (p) => { if (p.id === clip.id) setProgress(p.pct); });

  // preview: keep playback inside the trimmed range
  useEffect(() => {
    const el = v.current;
    if (!el) return;
    const onTime = () => { if (el.currentTime * 1000 >= range[1]) { el.pause(); el.currentTime = range[0] / 1000; } };
    el.addEventListener("timeupdate", onTime);
    return () => el.removeEventListener("timeupdate", onTime);
  }, [range]);
  useEffect(() => { if (v.current) { v.current.muted = mute; v.current.volume = Math.min(1, volume / 100); } }, [mute, volume]);

  const cropRect = useMemo(() => cropFor(crop, clip.width, clip.height), [crop, clip.width, clip.height]);
  const textY = textPos === "top" ? 0.06 : textPos === "center" ? 0.46 : 0.86;

  const setStart = (ms: number) => { const s = Math.min(ms, range[1] - 500); setRange([s, range[1]]); if (v.current) v.current.currentTime = s / 1000; };
  const setEnd = (ms: number) => setRange([range[0], Math.max(ms, range[0] + 500)]);

  const run = async () => {
    setError(null); setDone(false); setProgress(0);
    const opts: ExportOpts = { id: clip.id, startMs: range[0], endMs: range[1], crop: cropRect, rotate, mute, volumePct: volume, text: text.trim() ? { text: text.trim(), x: 0.05, y: textY, size: Math.round(clip.height / 20) } : null };
    try { const out = await ipc.exportClip(opts); setDone(true); setProgress(100); onDone(out); }
    catch (e) { setError(normalizeError(e)); setProgress(null); }
  };

  const changed = range[0] > 0 || range[1] < total || crop !== "none" || rotate !== 0 || mute || volume !== 100 || !!text.trim();
  const style = cropRect ? { clipPath: `inset(${(cropRect.y / clip.height) * 100}% ${(1 - (cropRect.x + cropRect.w) / clip.width) * 100}% ${(1 - (cropRect.y + cropRect.h) / clip.height) * 100}% ${(cropRect.x / clip.width) * 100}%)` } : undefined;

  return (
    <div className="editor">
      <div className="video-box editor-video">
        <video ref={v} src={src} controls style={{ ...style, transform: `rotate(${rotate}deg)` }} onLoadedMetadata={() => { if (v.current) v.current.currentTime = range[0] / 1000; }} />
        {text.trim() && <div className="text-preview" style={{ top: `${textY * 100}%` }}>{text}</div>}
      </div>

      <div className="card trim">
        <div className="card-title"><Scissors size={14} />{t("editor.trim")}</div>
        <div className="trim-track" aria-hidden>
          <div className="trim-sel" style={{ insetInlineStart: `${(range[0] / total) * 100}%`, insetInlineEnd: `${100 - (range[1] / total) * 100}%` }} />
        </div>
        <div className="trim-inputs">
          <input type="range" className="slider" aria-label={t("editor.start")} min={0} max={total} step={100} value={range[0]} onChange={(e) => setStart(Number(e.target.value))} style={{ ["--p" as string]: "0%" }} />
          <input type="range" className="slider" aria-label={t("editor.end")} min={0} max={total} step={100} value={range[1]} onChange={(e) => setEnd(Number(e.target.value))} style={{ ["--p" as string]: "100%" }} />
        </div>
        <div className="row-gap between mono num"><span>{formatDuration(range[0])}</span><span className="dim">{t("editor.length", { len: formatDuration(range[1] - range[0]) })}</span><span>{formatDuration(range[1])}</span></div>
      </div>

      <div className="card">
        <Field label={t("editor.crop")}>
          <Segmented<CropPreset> label={t("editor.crop")} value={crop} onChange={setCrop} options={(["none", "16:9", "4:3", "1:1", "9:16"] as CropPreset[]).map((c) => ({ value: c, label: c === "none" ? t("editor.none") : c }))} />
        </Field>
        <Field label={t("editor.rotate")}>
          <button className="btn" onClick={() => setRotate((r) => (r + 90) % 360)}><RotateCw size={15} />{rotate}°</button>
        </Field>
        <Field label={t("editor.mute")}><Toggle label={t("editor.mute")} checked={mute} onChange={setMute} />{mute ? <VolumeX size={16} className="faint" /> : <Volume2 size={16} className="dim" />}</Field>
        <Field label={t("editor.volume")}><Slider label={t("editor.volume")} value={volume} min={0} max={200} onChange={setVolume} format={(x) => `${x}%`} /></Field>
        <Field label={t("editor.text")} hint={t("editor.textHint")} wide>
          <input className="input" aria-label={t("editor.text")} placeholder={t("editor.textPlaceholder")} value={text} onChange={(e) => setText(e.target.value)} />
          <Segmented<"top" | "center" | "bottom"> label={t("editor.textPos")} value={textPos} onChange={setTextPos} options={[{ value: "top", label: t("editor.top") }, { value: "center", label: t("editor.center") }, { value: "bottom", label: t("editor.bottom") }]} />
        </Field>
      </div>

      {error && <ErrorBanner error={error} onDismiss={() => setError(null)} />}
      <div className="row-gap between">
        <div style={{ flex: 1, maxWidth: 360 }}>{progress !== null && <><Progress value={progress} /><div className="dim num" style={{ fontSize: 12, marginTop: 4 }}>{done ? t("editor.done") : `${Math.round(progress)}%`}</div></>}</div>
        <button className="btn primary" disabled={!changed || (progress !== null && !done && !error)} onClick={() => void run()}>{done ? <Check size={16} /> : <Type size={16} style={{ display: "none" }} />}{t("editor.export")}</button>
      </div>
      <p className="dim" style={{ fontSize: 12.5 }}>{t("editor.nonDestructive")}</p>
    </div>
  );
}

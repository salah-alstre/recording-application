import { useEffect, useState } from "react";
import { Film, Image as ImageIcon, MoreHorizontal, Play, RotateCcw, Star } from "lucide-react";
import { useT, useLang } from "@/i18n";
import type { Clip } from "@/lib/types";
import { assetUrl } from "@/hooks/hooks";
import { formatBytes, formatDate, formatDuration, resolutionLabel } from "@/lib/format";
import { Chip } from "@/components/feedback";
import { useLibrary } from "@/stores/library";

/** Real cover image. Pending → shimmer, broken/failed → neutral fallback; the browser's broken-image icon is never shown. */
export function ClipThumb({ clip }: { clip: Clip }) {
  const busy = useLibrary((s) => s.thumbsBusy);
  const [failed, setFailed] = useState(false);
  useEffect(() => setFailed(false), [clip.thumbPath]);
  const src = assetUrl(clip.thumbPath);
  const pending = !clip.thumbPath && busy;
  return (
    <div className="thumb">
      {src && !failed ? (
        <img src={src} alt="" loading="lazy" decoding="async" draggable={false} onError={() => setFailed(true)} />
      ) : pending ? (
        <div className="thumb-loading skeleton" aria-hidden />
      ) : (
        <div className="thumb-fallback">{clip.kind === "screenshot" ? <ImageIcon size={26} /> : <Film size={26} />}</div>
      )}
      {clip.kind !== "screenshot" && <span className="thumb-dur mono num">{formatDuration(clip.durationMs)}</span>}
      {clip.kind === "replay" && <span className="thumb-kind"><RotateCcw size={12} /></span>}
      {clip.kind === "screenshot" && <span className="thumb-kind"><ImageIcon size={12} /></span>}
    </div>
  );
}

export function ClipCard({ clip, onOpen, onFavorite, onMenu }: { clip: Clip; onOpen: () => void; onFavorite: () => void; onMenu: (at: { x: number; y: number }) => void }) {
  const t = useT();
  const lang = useLang((s) => s.lang);
  return (
    <article className="clip" onContextMenu={(e) => { e.preventDefault(); onMenu({ x: e.clientX, y: e.clientY }); }}>
      <button className="clip-open" onClick={onOpen} aria-label={`${t("library.play")}: ${clip.title}`}>
        <ClipThumb clip={clip} />
        <span className="clip-play" aria-hidden><Play size={22} fill="currentColor" /></span>
      </button>
      <div className="clip-actions">
        <button className={`btn icon sm clip-star${clip.favorite ? " on" : ""}`} aria-pressed={clip.favorite} aria-label={t("library.favorite")} onClick={onFavorite}><Star size={15} fill={clip.favorite ? "currentColor" : "none"} /></button>
        <button className="btn icon sm" aria-label={t("common.more")} onClick={(e) => { const r = e.currentTarget.getBoundingClientRect(); onMenu({ x: r.left, y: r.bottom + 4 }); }}><MoreHorizontal size={15} /></button>
      </div>
      <div className="clip-body">
        <div className="clip-title" title={clip.title}>{clip.title}</div>
        <div className="dim clip-sub"><span>{clip.game || t("library.desktop")}</span><span aria-hidden>·</span><span>{formatDate(clip.createdAt, lang)}</span></div>
        <div className="chips">
          {clip.kind !== "screenshot" && <Chip>{formatDuration(clip.durationMs)}</Chip>}
          <Chip>{clip.kind === "screenshot" ? `${clip.width}×${clip.height}` : `${resolutionLabel(clip.height)}${clip.fps ? ` · ${Math.round(clip.fps)} fps` : ""}`}</Chip>
          <Chip>{formatBytes(clip.sizeBytes)}</Chip>
        </div>
      </div>
    </article>
  );
}

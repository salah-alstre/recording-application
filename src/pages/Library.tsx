import { useEffect, useState } from "react";
import { ArrowDownUp, Clapperboard, Copy, FolderOpen, Pencil, Play, RefreshCw, Search, Star, Trash2 } from "lucide-react";
import { useT } from "@/i18n";
import { useLibrary } from "@/stores/library";
import { useUi } from "@/stores/ui";
import { useDebounced } from "@/hooks/hooks";
import { Segmented, Select } from "@/components/controls";
import { ContextMenu, Dialog, type MenuEntry } from "@/components/overlays";
import { EmptyState, Skeleton } from "@/components/feedback";
import { ClipCard } from "@/features/library/ClipCard";
import { Player } from "@/features/library/Player";
import { ipc } from "@/lib/ipc";
import { attempt } from "@/features/common/actions";
import type { Clip } from "@/lib/types";

export function Library() {
  const t = useT();
  const { clips, loaded, loading, query, setQuery, scan } = useLibrary();
  const playerId = useUi((s) => s.playerClipId);
  const play = useUi((s) => s.play);
  const [search, setSearch] = useState(query.search);
  const debounced = useDebounced(search, 220);
  const [menu, setMenu] = useState<{ at: { x: number; y: number }; clip: Clip } | null>(null);
  const [renaming, setRenaming] = useState<Clip | null>(null);
  const [deleting, setDeleting] = useState<Clip | null>(null);
  const [name, setName] = useState("");

  useEffect(() => { if (debounced !== query.search) setQuery({ search: debounced }); }, [debounced, query.search, setQuery]);
  useEffect(() => { void scan(); }, [scan]);

  if (playerId) return <Player id={playerId} onClose={() => play(null)} />;

  const entries = (c: Clip): MenuEntry[] => [
    { label: t("library.play"), icon: <Play size={15} />, onClick: () => play(c.id) },
    { label: t("library.rename"), icon: <Pencil size={15} />, onClick: () => { setName(c.title); setRenaming(c); } },
    { label: c.favorite ? t("library.unfavorite") : t("library.favorite"), icon: <Star size={15} />, onClick: () => void ipc.setFavorite(c.id, !c.favorite).then(() => useLibrary.getState().refresh()) },
    { label: t("library.reveal"), icon: <FolderOpen size={15} />, onClick: () => void ipc.revealClip(c.id).catch(() => undefined) },
    { label: t("library.copyPath"), icon: <Copy size={15} />, onClick: () => void navigator.clipboard.writeText(c.path) },
    { label: t("library.delete"), icon: <Trash2 size={15} />, danger: true, separatorBefore: true, onClick: () => setDeleting(c) },
  ];

  const sections = [
    { value: "all", label: t("library.all") }, { value: "recording", label: t("library.recordings") }, { value: "replay", label: t("library.replays") },
    { value: "screenshot", label: t("library.screenshots") }, { value: "favorite", label: t("library.favorites") },
  ];
  const sorts = ["newest", "oldest", "largest", "smallest", "longest", "shortest"].map((v) => ({ value: v, label: t(`sort.${v}` as "sort.newest") }));

  return (
    <div className="page">
      <div className="page-head">
        <h1 className="page-title">{t("nav.library")}</h1>
        <div className="lib-tools">
          <label className="search"><Search size={16} className="faint" /><input className="input" placeholder={t("library.search")} aria-label={t("library.search")} value={search} onChange={(e) => setSearch(e.target.value)} /></label>
          <div style={{ width: 170 }}><Select label={t("library.sort")} value={query.sort} options={sorts} onChange={(v) => setQuery({ sort: v })} /></div>
          <button className="btn icon" aria-label={t("library.rescan")} title={t("library.rescan")} onClick={() => void scan()}><RefreshCw size={16} className={loading ? "spin" : ""} /></button>
        </div>
      </div>
      <div style={{ marginBottom: 18 }}><Segmented label={t("nav.library")} value={query.section === "favorites" ? "favorite" : query.section} options={sections} onChange={(v) => setQuery({ section: v === "favorite" ? "favorites" : v })} /></div>

      {!loaded ? (
        <div className="clip-grid">{Array.from({ length: 8 }).map((_, i) => <Skeleton key={i} h={250} r={18} />)}</div>
      ) : clips.length === 0 ? (
        <EmptyState icon={<Clapperboard size={34} />} title={query.search || query.section !== "all" ? t("library.noResults") : t("library.emptyTitle")} body={query.search || query.section !== "all" ? t("library.noResultsBody") : t("library.emptyBody")} />
      ) : (
        <div className="clip-grid">
          {clips.map((c) => (
            <div key={c.id} className="page-enter">
              <ClipCard clip={c} onOpen={() => play(c.id)} onFavorite={() => void ipc.setFavorite(c.id, !c.favorite).then(() => useLibrary.getState().refresh())} onMenu={(at) => setMenu({ at, clip: c })} />
            </div>
          ))}
        </div>
      )}
      <span className="sr-only"><ArrowDownUp /></span>

      <ContextMenu at={menu?.at ?? null} entries={menu ? entries(menu.clip) : []} onClose={() => setMenu(null)} />

      <Dialog open={!!renaming} onClose={() => setRenaming(null)} title={t("library.rename")}
        actions={<><button className="btn" onClick={() => setRenaming(null)}>{t("common.cancel")}</button>
          <button className="btn primary" disabled={!name.trim()} onClick={() => { const c = renaming!; setRenaming(null); void attempt(ipc.renameClip(c.id, name)).then(() => useLibrary.getState().refresh()); }}>{t("common.save")}</button></>}>
        <input className="input" autoFocus aria-label={t("library.rename")} value={name} onChange={(e) => setName(e.target.value)} onKeyDown={(e) => { if (e.key === "Enter" && name.trim()) { const c = renaming!; setRenaming(null); void attempt(ipc.renameClip(c.id, name)).then(() => useLibrary.getState().refresh()); } }} />
      </Dialog>

      <Dialog open={!!deleting} onClose={() => setDeleting(null)} title={t("library.deleteTitle")}
        actions={<><button className="btn" onClick={() => setDeleting(null)}>{t("common.cancel")}</button>
          <button className="btn danger" onClick={() => { const c = deleting!; setDeleting(null); void attempt(ipc.deleteClip(c.id)).then(() => useLibrary.getState().refresh()); }}><Trash2 size={15} />{t("library.delete")}</button></>}>
        <p className="dim">{t("library.deleteBody", { name: deleting?.title ?? "" })}</p>
      </Dialog>
    </div>
  );
}

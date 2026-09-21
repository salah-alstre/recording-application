import { useEffect, useState, type ReactNode } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { AnimatePresence, motion } from "motion/react";
import { Activity, Clapperboard, Home as HomeIcon, Layers, Minus, Settings as SettingsIcon, Square, X, Copy } from "lucide-react";
import { Logo } from "@/components/Logo";
import { useT } from "@/i18n";
import { useUi, type Page } from "@/stores/ui";
import { useRecording, elapsedMs, isRecording } from "@/stores/recording";
import { useNow } from "@/hooks/hooks";
import { formatClock } from "@/lib/format";
import { LanguageSwitch } from "@/features/settings/LanguageSwitch";

export function TitleBar() {
  const t = useT();
  const [max, setMax] = useState(false);
  const w = getCurrentWindow();
  useEffect(() => {
    const sync = () => void w.isMaximized().then(setMax);
    sync();
    const p = w.onResized(sync);
    return () => void p.then((u) => u());
  }, [w]);
  return (
    <header className="titlebar" data-tauri-drag-region onDoubleClick={() => void w.toggleMaximize()}>
      <div className="tb-brand" data-tauri-drag-region>
        <Logo size={20} />
        <span className="tb-name" data-tauri-drag-region>Rimlight</span>
      </div>
      <div className="tb-actions">
        <button className="tb-btn" aria-label={t("window.minimize")} onClick={() => void w.minimize()}><Minus size={15} /></button>
        <button className="tb-btn" aria-label={t("window.maximize")} onClick={() => void w.toggleMaximize()}>{max ? <Copy size={13} /> : <Square size={12} />}</button>
        <button className="tb-btn close" aria-label={t("window.close")} onClick={() => void w.close()}><X size={16} /></button>
      </div>
    </header>
  );
}

const NAV: { page: Page; icon: ReactNode; key: "nav.home" | "nav.library" | "nav.profiles" | "nav.diagnostics" | "nav.settings" }[] = [
  { page: "home", icon: <HomeIcon size={19} />, key: "nav.home" },
  { page: "library", icon: <Clapperboard size={19} />, key: "nav.library" },
  { page: "profiles", icon: <Layers size={19} />, key: "nav.profiles" },
  { page: "diagnostics", icon: <Activity size={19} />, key: "nav.diagnostics" },
  { page: "settings", icon: <SettingsIcon size={19} />, key: "nav.settings" },
];

export function Sidebar() {
  const t = useT();
  const { page, go } = useUi();
  const status = useRecording((s) => s.status);
  const rec = isRecording(status);
  const now = useNow(rec && status.state === "recording", 500);
  return (
    <nav className="sidebar" aria-label={t("nav.label")}>
      <ul>
        {NAV.map((n) => (
          <li key={n.page}>
            <button className="nav-item" aria-current={page === n.page ? "page" : undefined} onClick={() => go(n.page)}>
              {page === n.page && <span className="nav-active" />}
              <span className="nav-ico">{n.icon}</span>
              <span className="nav-label">{t(n.key)}</span>
            </button>
          </li>
        ))}
      </ul>
      <div className="sidebar-foot">
        <AnimatePresence initial={false}>
          {(rec || status.replay) && (
            <motion.div className="side-status" initial={{ opacity: 0, height: 0 }} animate={{ opacity: 1, height: "auto" }} exit={{ opacity: 0, height: 0 }}>
              {rec && (
                <div className="side-row"><span className={`dot ${status.state === "recording" ? "rec" : ""}`} /><span className="mono num">{formatClock(elapsedMs(status, now))}</span></div>
              )}
              {status.replay && <div className="side-row"><span className="dot ok" /><span>{t("replay.on")}</span></div>}
            </motion.div>
          )}
        </AnimatePresence>
        <LanguageSwitch compact />
      </div>
    </nav>
  );
}

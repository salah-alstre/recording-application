import { useEffect, useRef, useState } from "react";
import { TitleBar, Sidebar } from "@/features/shell/Shell";
import { Home } from "@/pages/Home";
import { Library } from "@/pages/Library";
import { Profiles } from "@/pages/Profiles";
import { Diagnostics } from "@/pages/Diagnostics";
import { Settings } from "@/pages/Settings";
import { Onboarding } from "@/features/onboarding/Onboarding";
import { Dialog } from "@/components/overlays";
import { ErrorBoundary } from "@/components/PageBoundary";
import { navlog } from "@/lib/navlog";
import { useSettings } from "@/stores/settings";
import { useUi, type Page } from "@/stores/ui";
import { useT } from "@/i18n";
import { useTauriEvent } from "@/hooks/hooks";
import { subscribeLibrary } from "@/stores/library";
import { ipc } from "@/lib/ipc";
import { formatDate } from "@/lib/format";
import { useLang } from "@/i18n";

const PAGES: Record<Page, () => JSX.Element> = { home: Home, library: Library, profiles: Profiles, diagnostics: Diagnostics, settings: Settings };

function Recovery() {
  const t = useT();
  const lang = useLang((s) => s.lang);
  const { interrupted, setInterrupted } = useUi();
  const [busy, setBusy] = useState(false);
  const row = interrupted[0];
  const next = () => setInterrupted(interrupted.slice(1));
  return (
    <Dialog open={!!row} onClose={next} title={t("recovery.title")}
      actions={<>
        <button className="btn" disabled={busy} onClick={() => { const r = row; setBusy(true); void ipc.discardSession(r.id).finally(() => { setBusy(false); next(); }); }}>{t("recovery.discard")}</button>
        <button className="btn primary" disabled={busy} onClick={() => { const r = row; setBusy(true); void ipc.recoverSession(r.id).catch(() => undefined).finally(() => { setBusy(false); next(); void subscribeLibrary; }); }}>{busy ? t("recovery.recovering") : t("recovery.recover")}</button></>}>
      {row && <p className="dim">{t("recovery.body", { game: row.game || t("library.desktop"), date: formatDate(row.startedAt, lang) })}</p>}
    </Dialog>
  );
}

export function App() {
  const settings = useSettings((s) => s.settings);
  const page = useUi((s) => s.page);
  const go = useUi((s) => s.go);
  const setInterrupted = useUi((s) => s.setInterrupted);
  const [onboarding, setOnboarding] = useState(false);
  const Page = PAGES[page] ?? Home;

  const prev = useRef<Page>(page);
  useEffect(() => {
    if (prev.current !== page) navlog("From:", prev.current, "To:", page);
    navlog("Current route:", page, "Component:", (PAGES[page] ?? Home).name);
    prev.current = page;
  }, [page]);
  useEffect(() => { navlog("Render completed:", page); });

  useTauriEvent<string>("navigate", (p) => go(p in PAGES ? (p as Page) : "home"));
  useEffect(() => { void ipc.listInterrupted().then(setInterrupted).catch(() => undefined); }, [setInterrupted]);
  useEffect(() => { if (settings && !settings.general.onboarded) setOnboarding(true); }, [settings]);

  return (
    <div className="app">
      <TitleBar />
      <div className="app-body">
        <Sidebar />
        <main className="content" id="content">
          {/* Rendering never waits on an animation: the page mounts synchronously and only a
              transform-only CSS entrance runs, so content stays visible even if frames stall. */}
          <div key={page} className="content-inner page-enter">
            <ErrorBoundary name={page}><Page /></ErrorBoundary>
          </div>
        </main>
      </div>
      {onboarding && settings && <Onboarding onFinish={() => setOnboarding(false)} />}
      {!onboarding && <Recovery />}
    </div>
  );
}

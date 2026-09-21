import { useEffect } from "react";
import { AppWindow, Crop, Gamepad2, Monitor, MousePointer2, RefreshCw } from "lucide-react";
import { Select, Segmented, type Option } from "@/components/controls";
import { InfoBanner } from "@/components/overlays";
import { useT } from "@/i18n";
import { useSettings } from "@/stores/settings";
import { useDevices } from "@/stores/devices";
import { useRecording } from "@/stores/recording";
import { ipc } from "@/lib/ipc";
import type { Settings } from "@/lib/types";
import { resolutionLabel } from "@/lib/format";

type Mode = Settings["capture"]["mode"];

export function MonitorCards({ compact }: { compact?: boolean }) {
  const t = useT();
  const monitors = useDevices((s) => s.monitors);
  const idx = useSettings((s) => s.settings?.capture.displayIndex ?? 1);
  const patch = useSettings((s) => s.patch);
  return (
    <div className={`monitor-grid${compact ? " compact" : ""}`} role="radiogroup" aria-label={t("source.display")}>
      {monitors.map((m) => (
        <button key={m.index} role="radio" aria-checked={idx === m.index} className="monitor-card" onClick={() => patch((d) => { d.capture.displayIndex = m.index; }, { immediate: true })}>
          <div className="monitor-art" aria-hidden><Monitor size={compact ? 18 : 22} /><span className="num">{m.index}</span></div>
          <div className="monitor-meta">
            <div className="monitor-name">{t("source.displayN", { n: m.index })}{m.primary && <span className="chip accent" style={{ marginInlineStart: 8 }}>{t("source.primary")}</span>}</div>
            <div className="dim num">{m.width}×{m.height} · {m.refreshHz} Hz</div>
          </div>
        </button>
      ))}
    </div>
  );
}

export function SourceCard() {
  const t = useT();
  const s = useSettings((x) => x.settings);
  const patch = useSettings((x) => x.patch);
  const windows = useDevices((x) => x.windows);
  const loadWindows = useDevices((x) => x.loadWindows);
  const loadMonitors = useDevices((x) => x.loadMonitors);
  const game = useRecording((x) => x.status.game);
  const mode = s?.capture.mode ?? "display";

  useEffect(() => {
    void loadMonitors();
    if (mode === "window") void loadWindows();
  }, [mode, loadMonitors, loadWindows]);

  if (!s) return null;
  const modes: Option<Mode>[] = [
    { value: "display", label: t("source.display"), icon: <Monitor size={15} /> },
    { value: "window", label: t("source.window"), icon: <AppWindow size={15} /> },
    { value: "game", label: t("source.game"), icon: <Gamepad2 size={15} /> },
    { value: "region", label: t("source.region"), icon: <Crop size={15} /> },
    { value: "active", label: t("source.active"), icon: <MousePointer2 size={15} /> },
  ];
  const winOptions = windows.map((w) => ({ value: `${w.exe}|${w.title}`, label: w.title.length > 54 ? w.title.slice(0, 54) + "…" : w.title, hint: w.exe }));
  const curWin = `${s.capture.windowExe}|${s.capture.windowTitle}`;
  if (s.capture.windowExe && !winOptions.some((o) => o.value === curWin)) winOptions.unshift({ value: curWin, label: s.capture.windowTitle, hint: s.capture.windowExe });

  return (
    <section className="card source" aria-labelledby="source-title">
      <div className="card-title" id="source-title"><Monitor size={15} />{t("source.title")}</div>
      <Segmented<Mode> label={t("source.title")} value={mode} options={modes} onChange={(v) => patch((d) => { d.capture.mode = v; }, { immediate: true })} />
      <div className="source-body">
        {mode === "display" && <MonitorCards />}
        {mode === "window" && (
          <div className="row-gap">
            <Select label={t("source.window")} value={curWin} options={winOptions} placeholder={t("source.pickWindow")}
              onChange={(v) => { const w = windows.find((x) => `${x.exe}|${x.title}` === v); patch((d) => { d.capture.windowExe = w?.exe ?? v.split("|")[0]; d.capture.windowTitle = w?.title ?? v.split("|").slice(1).join("|"); }, { immediate: true }); }} />
            <button className="btn icon" aria-label={t("common.refresh")} onClick={() => void loadWindows()}><RefreshCw size={16} /></button>
          </div>
        )}
        {mode === "game" && (
          game ? (
            <div className="game-card">
              <Gamepad2 size={22} className="accent-text" />
              <div style={{ minWidth: 0 }}>
                <div className="game-name">{game.name}</div>
                <div className="dim num">{game.exe} · {game.width}×{game.height} · {t("source.displayN", { n: game.displayIndex })} · {game.fullscreen ? t("source.fullscreen") : t("source.windowed")}</div>
              </div>
            </div>
          ) : <InfoBanner>{t("source.noGame")}</InfoBanner>
        )}
        {mode === "region" && (
          <div className="row-gap">
            <div className="game-card" style={{ flex: 1 }}>
              <Crop size={22} className="accent-text" />
              <div><div className="game-name num">{s.capture.region.w}×{s.capture.region.h}</div><div className="dim">{t("source.displayN", { n: s.capture.displayIndex })} · {resolutionLabel(s.capture.region.h)}</div></div>
            </div>
            <button className="btn primary" onClick={() => void ipc.regionPick()}><Crop size={16} />{t("source.selectRegion")}</button>
          </div>
        )}
        {mode === "active" && <InfoBanner>{t("source.activeHint")}</InfoBanner>}
      </div>
    </section>
  );
}

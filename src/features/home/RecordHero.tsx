import { motion } from "motion/react";
import { Camera, Clapperboard, Pause, Play, RotateCcw, Flag } from "lucide-react";
import { useT } from "@/i18n";
import { useRecording, elapsedMs, isRecording } from "@/stores/recording";
import { useUi } from "@/stores/ui";
import { useNow } from "@/hooks/hooks";
import { formatClock, resolutionLabel } from "@/lib/format";
import { actions } from "@/features/common/actions";
import { Chip } from "@/components/feedback";
import { useSettings } from "@/stores/settings";
import { KeyCaps } from "@/components/controls";

export function RecordButton({ size = 132 }: { size?: number }) {
  const t = useT();
  const state = useRecording((s) => s.status.state);
  const active = state === "recording" || state === "paused";
  const busy = state === "preparing" || state === "stopping";
  return (
    <button className="rec-btn" style={{ width: size, height: size }} data-state={state} disabled={busy}
      aria-label={active ? t("rec.stop") : t("rec.start")} aria-pressed={active} onClick={() => void actions.toggleRecord()}>
      <svg className="rec-ring" viewBox="0 0 100 100" aria-hidden>
        <circle cx="50" cy="50" r="46" fill="none" strokeWidth="2" />
        <circle className="rec-sweep" cx="50" cy="50" r="46" fill="none" strokeWidth="3.5" strokeLinecap="round" />
      </svg>
      <motion.span className="rec-core" animate={{ width: active ? size * 0.26 : size * 0.4, height: active ? size * 0.26 : size * 0.4, borderRadius: active ? size * 0.07 : size * 0.4 }}
        transition={{ type: "spring", stiffness: 380, damping: 26 }} />
    </button>
  );
}

export function RecordHero() {
  const t = useT();
  const status = useRecording((s) => s.status);
  const hotkeys = useSettings((s) => s.settings?.hotkeys);
  const rec = isRecording(status);
  const now = useNow(status.state === "recording", 250);
  const go = useUi((s) => s.go);
  const label = { idle: t("rec.ready"), preparing: t("rec.preparing"), recording: t("rec.recording"), paused: t("rec.paused"), stopping: t("rec.stopping"), error: t("rec.error") }[status.state];
  const r = status.recording;
  return (
    <section className="card hero" aria-labelledby="hero-title">
      <div className="hero-glow" aria-hidden />
      <div className="hero-main">
        <RecordButton />
        <div className="hero-text">
          <div className="hero-state"><span className={`dot ${status.state === "recording" ? "rec" : status.state === "paused" ? "" : ""}`} /><span id="hero-title">{label}</span></div>
          <div className={`hero-timer mono num${status.state === "paused" ? " paused" : ""}`}>{formatClock(rec ? elapsedMs(status, now) : 0)}</div>
          <div className="hero-chips">
            {r ? (
              <>
                <Chip tone="accent">{r.encoder}</Chip>
                <Chip>{resolutionLabel(r.height)} · {r.fps} fps</Chip>
                {r.game && <Chip>{r.game}</Chip>}
              </>
            ) : (
              <span className="dim">{hotkeys?.record && <>{t("hero.press")} <KeyCaps accel={hotkeys.record} /> {t("hero.toStart")}</>}</span>
            )}
          </div>
        </div>
      </div>
      <div className="hero-actions">
        {rec ? (
          <>
            <button className="btn" onClick={() => void (status.state === "paused" ? actions.resume() : actions.pause())}>
              {status.state === "paused" ? <Play size={16} /> : <Pause size={16} />}{status.state === "paused" ? t("rec.resume") : t("rec.pause")}
            </button>
            <button className="btn" onClick={() => void actions.marker()}><Flag size={16} />{t("rec.marker")}</button>
          </>
        ) : (
          <button className="btn" disabled={!!status.replay?.saving || !status.replay} onClick={() => void actions.saveReplay()}><RotateCcw size={16} />{t("replay.save")}</button>
        )}
        <button className="btn" onClick={() => void actions.screenshot()}><Camera size={16} />{t("shot.take")}</button>
        <button className="btn" onClick={() => go("library")}><Clapperboard size={16} />{t("home.openLibrary")}</button>
      </div>
    </section>
  );
}

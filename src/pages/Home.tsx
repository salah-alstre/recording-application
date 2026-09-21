import { useEffect } from "react";
import { RecordHero } from "@/features/home/RecordHero";
import { SourceCard } from "@/features/home/SourceCard";
import { AudioCard, EncoderCard, ProfileCard, RecentClips, ReplayCard, StorageCard } from "@/features/home/Cards";
import { ErrorBanner } from "@/components/overlays";
import { useRecording } from "@/stores/recording";
import { useLibrary } from "@/stores/library";
import { useUi } from "@/stores/ui";
import { ipc } from "@/lib/ipc";
import { useT } from "@/i18n";

export function Home() {
  const t = useT();
  const err = useRecording((s) => s.status.error);
  const refresh = useRecording((s) => s.refresh);
  const loadClips = useLibrary((s) => s.refresh);
  useEffect(() => { void loadClips(); }, [loadClips]);
  const uiErr = useUi((s) => s.error);
  const setUiErr = useUi((s) => s.setError);
  return (
    <div className="page">
      <h1 className="page-title">{t("nav.home")}</h1>
      {(err || uiErr) && (
        <div style={{ marginBottom: 16 }}>
          <ErrorBanner error={(err ?? uiErr)!} onDismiss={() => { setUiErr(null); void ipc.dismissError().then(() => refresh()); }} />
        </div>
      )}
      <div className="bento">
        <div className="span-7"><RecordHero /></div>
        <div className="span-5"><SourceCard /></div>
        <div className="span-4"><ReplayCard /></div>
        <div className="span-4"><AudioCard /></div>
        <div className="span-4"><EncoderCard /></div>
        <div className="span-4"><ProfileCard /></div>
        <div className="span-4"><StorageCard /></div>
        <div className="span-4"><RecentClips /></div>
      </div>
    </div>
  );
}

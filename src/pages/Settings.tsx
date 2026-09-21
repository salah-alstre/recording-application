import type { ReactNode } from "react";
import { AppWindow, Camera, Gauge, Globe, HardDrive, Keyboard, LayoutPanelTop, Mic, Palette, RotateCcw, Settings2, ShieldCheck, SlidersHorizontal, Video, Volume2, Wrench } from "lucide-react";
import { useT } from "@/i18n";
import { useUi, type SettingsSection } from "@/stores/ui";
import { useSettings } from "@/stores/settings";
import { AudioSection, CameraSection, General, Microphone, Recording, Replay } from "@/features/settings/sectionsA";
import { Advanced, Appearance, Hotkeys, LanguageSection, OverlaySection, PerformanceSection, PrivacySection, Screenshots, StorageSection } from "@/features/settings/sectionsB";

const SECTIONS: { id: SettingsSection; icon: ReactNode; render: () => ReactNode }[] = [
  { id: "general", icon: <Settings2 size={17} />, render: () => <General /> },
  { id: "recording", icon: <Video size={17} />, render: () => <Recording /> },
  { id: "replay", icon: <RotateCcw size={17} />, render: () => <Replay /> },
  { id: "audio", icon: <Volume2 size={17} />, render: () => <AudioSection /> },
  { id: "microphone", icon: <Mic size={17} />, render: () => <Microphone /> },
  { id: "camera", icon: <Camera size={17} />, render: () => <CameraSection /> },
  { id: "screenshots", icon: <AppWindow size={17} />, render: () => <Screenshots /> },
  { id: "overlay", icon: <LayoutPanelTop size={17} />, render: () => <OverlaySection /> },
  { id: "hotkeys", icon: <Keyboard size={17} />, render: () => <Hotkeys /> },
  { id: "storage", icon: <HardDrive size={17} />, render: () => <StorageSection /> },
  { id: "performance", icon: <Gauge size={17} />, render: () => <PerformanceSection /> },
  { id: "privacy", icon: <ShieldCheck size={17} />, render: () => <PrivacySection /> },
  { id: "language", icon: <Globe size={17} />, render: () => <LanguageSection /> },
  { id: "appearance", icon: <Palette size={17} />, render: () => <Appearance /> },
  { id: "advanced", icon: <Wrench size={17} />, render: () => <Advanced /> },
];

export function Settings() {
  const t = useT();
  const section = useUi((s) => s.settingsSection);
  const go = useUi((s) => s.go);
  const loaded = useSettings((s) => !!s.settings);
  const cur = SECTIONS.find((s) => s.id === section) ?? SECTIONS[0];
  return (
    <div className="page settings-page">
      <h1 className="page-title"><SlidersHorizontal size={0} />{t("nav.settings")}</h1>
      <div className="settings-layout">
        <nav className="settings-nav" aria-label={t("nav.settings")}>
          {SECTIONS.map((s) => (
            <button key={s.id} className="sub-item" aria-current={s.id === section ? "page" : undefined} onClick={() => go("settings", s.id)}>
              {s.icon}<span>{t(`settings.${s.id}` as "settings.general")}</span>
            </button>
          ))}
        </nav>
        <div className="settings-body">
          <h2 className="section-title first">{t(`settings.${cur.id}` as "settings.general")}</h2>
          <div key={cur.id} className="page-enter">
            {loaded ? cur.render() : <div className="skeleton" style={{ height: 260 }} />}
          </div>
        </div>
      </div>
    </div>
  );
}

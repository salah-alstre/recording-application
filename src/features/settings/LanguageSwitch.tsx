import { Segmented } from "@/components/controls";
import { useLang, useT } from "@/i18n";
import { useSettings } from "@/stores/settings";
import type { Lang } from "@/lib/types";

/** Switches language instantly (UI + direction) and persists the choice. */
export function LanguageSwitch({ compact }: { compact?: boolean }) {
  const t = useT();
  const lang = useLang((s) => s.lang);
  const patch = useSettings((s) => s.patch);
  const change = (l: Lang) => {
    useLang.getState().setLang(l);
    patch((d) => { d.general.language = l; }, { immediate: true });
  };
  return (
    <Segmented<Lang> label={t("settings.language")} value={lang} onChange={change}
      options={[{ value: "en", label: compact ? "EN" : "English" }, { value: "ar", label: compact ? "ع" : "العربية" }]} />
  );
}

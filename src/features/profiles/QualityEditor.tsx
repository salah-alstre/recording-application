import { Field, Segmented, Select, Slider } from "@/components/controls";
import { InfoBanner } from "@/components/overlays";
import { useT } from "@/i18n";
import type { Quality } from "@/lib/types";
import { useDevices } from "@/stores/devices";
import { availableCodecs, estimateBytes, PRESETS, recommendedBitrate } from "@/lib/encoders";
import { formatBytes } from "@/lib/format";

const HEIGHTS = [0, 720, 1080, 1440, 2160];
const FPS = [30, 60, 90, 120, 144];

/** Full recording-quality editor: presets, resolution, fps, bitrate, codec, encoder, rate control, container. */
export function QualityEditor({ q, onChange, tracks = 3 }: { q: Quality; onChange: (q: Quality) => void; tracks?: number }) {
  const t = useT();
  const hw = useDevices((s) => s.hardware);
  const codecs = hw ? availableCodecs(hw.encoders, q.encoder) : ["h264"];
  const set = (patch: Partial<Quality>) => onChange({ ...q, ...patch, preset: "custom" });
  const preset = (p: string) => {
    if (p === "custom") return onChange({ ...q, preset: "custom" });
    onChange({ ...q, ...PRESETS[p], preset: p as Quality["preset"] });
  };
  const estimate = estimateBytes(q, tracks, 10);
  const hwEncoders = (hw?.encoders ?? []).filter((e) => e.available);
  const vendors = ["auto", "nvenc", "amf", "qsv", "cpu"].filter((v) => v === "auto" || v === "cpu" || hwEncoders.some((e) => e.vendor === v));

  return (
    <div>
      <Field label={t("quality.preset")} wide>
        <Segmented<Quality["preset"]> label={t("quality.preset")} value={q.preset} onChange={preset}
          options={(["low", "medium", "high", "ultra", "custom"] as const).map((p) => ({ value: p, label: t(`quality.${p}` as "quality.low") }))} />
      </Field>
      <Field label={t("quality.resolution")}>
        <Select label={t("quality.resolution")} value={q.height} onChange={(v) => set({ height: v, bitrateKbps: recommendedBitrate(v, q.fps, q.codec) })}
          options={HEIGHTS.map((h) => ({ value: h, label: h === 0 ? t("quality.native") : h === 2160 ? "4K (2160p)" : `${h}p` }))} />
      </Field>
      <Field label={t("quality.fps")}>
        <Select label={t("quality.fps")} value={q.fps} onChange={(v) => set({ fps: v, bitrateKbps: recommendedBitrate(q.height, v, q.codec) })} options={FPS.map((f) => ({ value: f, label: `${f} fps` }))} />
      </Field>
      <Field label={t("quality.codec")} hint={t("quality.codecHint")}>
        <Select label={t("quality.codec")} value={q.codec} onChange={(v) => set({ codec: v, bitrateKbps: recommendedBitrate(q.height, q.fps, v) })}
          options={(["h264", "hevc", "av1"] as const).map((c) => ({ value: c, label: c === "h264" ? "H.264" : c === "hevc" ? "H.265 / HEVC" : "AV1", hint: codecs.includes(c) ? undefined : t("quality.cpuOnly") }))} />
      </Field>
      <Field label={t("quality.encoder")} hint={t("quality.encoderHint")}>
        <Select label={t("quality.encoder")} value={q.encoder} onChange={(v) => set({ encoder: v })}
          options={vendors.map((v) => ({ value: v as Quality["encoder"], label: t(`quality.enc.${v}` as "quality.enc.auto") }))} />
      </Field>
      <Field label={t("quality.rateControl")}>
        <Segmented<Quality["rateControl"]> label={t("quality.rateControl")} value={q.rateControl} onChange={(v) => set({ rateControl: v })}
          options={[{ value: "cbr", label: "CBR" }, { value: "vbr", label: "VBR" }, { value: "cqp", label: "CQP" }]} />
      </Field>
      {q.rateControl === "cqp" ? (
        <Field label={t("quality.cq")} hint={t("quality.cqHint")}><Slider label={t("quality.cq")} value={q.cq} min={10} max={40} onChange={(v) => set({ cq: v })} /></Field>
      ) : (
        <Field label={t("quality.bitrate")} hint={t("quality.bitrateHint", { rec: Math.round(recommendedBitrate(q.height, q.fps, q.codec) / 1000) })}>
          <Slider label={t("quality.bitrate")} value={q.bitrateKbps} min={1500} max={150000} step={500} onChange={(v) => set({ bitrateKbps: v })} format={(v) => `${(v / 1000).toFixed(1)} Mbps`} />
        </Field>
      )}
      <Field label={t("quality.speed")} hint={t("quality.speedHint")}>
        <Segmented<Quality["speed"]> label={t("quality.speed")} value={q.speed} onChange={(v) => set({ speed: v })}
          options={[{ value: "fast", label: t("quality.fast") }, { value: "balanced", label: t("quality.balanced") }, { value: "quality", label: t("quality.best") }]} />
      </Field>
      <Field label={t("quality.container")} hint={t("quality.containerHint")}>
        <Segmented<Quality["container"]> label={t("quality.container")} value={q.container} onChange={(v) => set({ container: v })} options={[{ value: "mp4", label: "MP4" }, { value: "mkv", label: "MKV" }]} />
      </Field>
      <Field label={t("quality.audioBitrate")}>
        <Slider label={t("quality.audioBitrate")} value={q.audioBitrateKbps} min={64} max={320} step={32} onChange={(v) => set({ audioBitrateKbps: v })} format={(v) => `${v} kbps`} />
      </Field>
      <InfoBanner>{t("quality.estimate", { size: formatBytes(estimate, 0), min: 10 })} · {t("quality.estimateNote")}</InfoBanner>
    </div>
  );
}

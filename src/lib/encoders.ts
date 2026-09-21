import type { EncoderInfo, Quality } from "./types";
import { estimatePerMinutes } from "./format";

/** Mirrors the backend's fallback order so the UI can show which encoder will actually be used. */
export function pickEncoder(q: Pick<Quality, "encoder" | "codec">, encoders: EncoderInfo[]): EncoderInfo | null {
  const order: Record<string, string[]> = {
    nvenc: ["nvenc", "amf", "qsv", "cpu"], amf: ["amf", "nvenc", "qsv", "cpu"], qsv: ["qsv", "nvenc", "amf", "cpu"], cpu: ["cpu"],
    auto: ["nvenc", "amf", "qsv", "cpu"],
  };
  for (const v of order[q.encoder] ?? order.auto) {
    const hit = encoders.find((e) => e.available && e.vendor === v && e.codec === q.codec);
    if (hit) return hit;
  }
  return null;
}

export function availableCodecs(encoders: EncoderInfo[], vendor: Quality["encoder"]): string[] {
  const set = new Set<string>();
  for (const e of encoders) if (e.available && (vendor === "auto" || e.vendor === vendor)) set.add(e.codec);
  return ["h264", "hevc", "av1"].filter((c) => set.has(c));
}

/** Storage estimate shown next to the quality controls (matches the backend `estimate_bytes` for CBR/VBR). */
export function estimateBytes(q: Quality, tracks: number, minutes: number): number {
  return estimatePerMinutes(q.bitrateKbps, q.audioBitrateKbps, tracks, minutes);
}

export function recommendedBitrate(height: number, fps: number, codec: string): number {
  const h = height || 1080;
  const pixels = (h * h * 16) / 9 / (1920 * 1080);
  const fpsF = Math.pow(fps / 60, 0.75);
  const codecF = codec === "hevc" ? 0.72 : codec === "av1" ? 0.6 : 1;
  const kbps = 18000 * pixels * fpsF * codecF;
  return Math.min(150000, Math.max(1500, Math.round(kbps / 500) * 500));
}

export const PRESETS: Record<string, { height: number; fps: number; bitrateKbps: number; cq: number }> = {
  low: { height: 720, fps: 30, bitrateKbps: 5000, cq: 30 },
  medium: { height: 1080, fps: 60, bitrateKbps: 12000, cq: 26 },
  high: { height: 1080, fps: 60, bitrateKbps: 20000, cq: 22 },
  ultra: { height: 1440, fps: 60, bitrateKbps: 45000, cq: 19 },
};

import { describe, expect, it } from "vitest";
import { availableCodecs, pickEncoder, recommendedBitrate } from "./encoders";
import type { EncoderInfo } from "./types";

const enc = (id: string, vendor: EncoderInfo["vendor"], codec: string, available: boolean): EncoderInfo => ({ id, vendor, codec, label: id, available });
const list = [enc("h264_nvenc", "nvenc", "h264", true), enc("av1_nvenc", "nvenc", "av1", true), enc("h264_amf", "amf", "h264", false), enc("libx264", "cpu", "h264", true), enc("libx265", "cpu", "hevc", true)];

describe("encoder selection", () => {
  it("prefers hardware for auto", () => {
    expect(pickEncoder({ encoder: "auto", codec: "h264" }, list)?.id).toBe("h264_nvenc");
    expect(pickEncoder({ encoder: "auto", codec: "av1" }, list)?.id).toBe("av1_nvenc");
  });
  it("falls back to CPU when hardware lacks the codec", () => {
    expect(pickEncoder({ encoder: "auto", codec: "hevc" }, list)?.id).toBe("libx265");
    expect(pickEncoder({ encoder: "amf", codec: "h264" }, list)?.id).toBe("h264_nvenc");
  });
  it("respects an explicit CPU choice", () => {
    expect(pickEncoder({ encoder: "cpu", codec: "h264" }, list)?.id).toBe("libx264");
  });
  it("lists only codecs that work", () => {
    expect(availableCodecs(list, "auto")).toEqual(["h264", "hevc", "av1"]);
    expect(availableCodecs(list, "amf")).toEqual([]);
  });
  it("matches the backend bitrate model", () => {
    expect(recommendedBitrate(1080, 60, "h264")).toBe(18000);
    expect(recommendedBitrate(1440, 60, "av1")).toBeLessThan(recommendedBitrate(1440, 60, "hevc"));
  });
});

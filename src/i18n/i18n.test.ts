import { describe, expect, it } from "vitest";
import { en } from "./en";
import { ar } from "./ar";
import { interpolate, translate, describeError } from "./index";

const params = (s: string) => Array.from(s.matchAll(/\{(\w+)\}/g)).map((m) => m[1]).sort();

describe("localisation", () => {
  it("has the same keys in English and Arabic", () => {
    expect(Object.keys(ar).sort()).toEqual(Object.keys(en).sort());
  });
  it("uses the same {placeholders} in every translation", () => {
    for (const k of Object.keys(en) as (keyof typeof en)[]) {
      expect(params(ar[k]), `placeholders of ${k}`).toEqual(params(en[k]));
    }
  });
  it("has no empty strings", () => {
    for (const [k, v] of Object.entries(ar)) expect(v.trim(), k).not.toBe("");
  });
  it("interpolates and tolerates missing params", () => {
    expect(interpolate("Hello {name}", { name: "Ada" })).toBe("Hello Ada");
    expect(interpolate("Hello {name}", {})).toBe("Hello ");
    expect(interpolate("No params {x}")).toBe("No params {x}");
  });
  it("translates per language", () => {
    expect(translate("en", "nav.home")).toBe("Home");
    expect(translate("ar", "nav.home")).toBe("الرئيسية");
    expect(translate("ar", "unit.minutes", { n: 3 })).toBe("3 د");
  });
  it("Arabic strings actually contain Arabic script (except brand / unit terms)", () => {
    const latinOnlyOk = new Set(["perf.m.fps", "toast.recording_saved.body", "toast.mic_disconnected.body", "toast.system_disconnected.body", "toast.mic_failed.body", "toast.system_failed.body", "quality.enc.nvenc", "quality.enc.amf", "quality.enc.qsv"]);
    for (const [k, v] of Object.entries(ar)) {
      if (latinOnlyOk.has(k)) continue;
      expect(/[؀-ۿ]/.test(v), `${k}: ${v}`).toBe(true);
    }
  });
  it("localises backend errors and falls back to the backend text for unknown codes", () => {
    expect(describeError("ar", { code: "disk_full", message: "x", hint: null }).title).toContain("500");
    expect(describeError("en", { code: "brand_new_code", message: "Custom message", hint: "Do this" })).toEqual({ title: "Custom message", hint: "Do this" });
    expect(describeError("ar", { code: "folder_unavailable", message: "x", hint: "y" }).hint).toContain("الإعدادات");
  });
});

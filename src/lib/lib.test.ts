import { describe, expect, it } from "vitest";
import { estimatePerMinutes, formatBytes, formatClock, formatDuration, percent, resolutionLabel } from "./format";
import { accelParts, eventToAccel, isAcceptable } from "./hotkey";

describe("format", () => {
  it("formats bytes", () => {
    expect(formatBytes(0)).toBe("0 B");
    expect(formatBytes(1536)).toBe("1.5 KB");
    expect(formatBytes(350 * 1024 * 1024)).toBe("350 MB");
    expect(formatBytes(1.2 * 1024 ** 3)).toBe("1.2 GB");
  });
  it("formats durations and the recording clock", () => {
    expect(formatDuration(83_000)).toBe("1:23");
    expect(formatDuration(3_725_000)).toBe("1:02:05");
    expect(formatClock(754_000)).toBe("00:12:34");
    expect(formatClock(-5)).toBe("00:00:00");
  });
  it("labels resolutions", () => {
    expect(resolutionLabel(2160)).toBe("4K");
    expect(resolutionLabel(1440)).toBe("1440p");
    expect(resolutionLabel(0)).toBe("—");
  });
  it("estimates storage like the UI promises", () => {
    // 8 Mbit/s video, no audio, 10 minutes = 600 MB (decimal)
    expect(estimatePerMinutes(8000, 0, 0, 10)).toBe(600_000_000);
  });
  it("computes percentages safely", () => {
    expect(percent(50, 200)).toBe(25);
    expect(percent(1, 0)).toBe(0);
    expect(percent(500, 200)).toBe(100);
  });
});

describe("hotkey capture", () => {
  const ev = (code: string, mods: Partial<Record<"ctrlKey" | "altKey" | "shiftKey" | "metaKey", boolean>> = {}) => ({
    code, ctrlKey: false, altKey: false, shiftKey: false, metaKey: false, ...mods,
  });
  it("builds accelerators from key events", () => {
    expect(eventToAccel(ev("KeyM", { ctrlKey: true, altKey: true }))).toBe("Ctrl+Alt+M");
    expect(eventToAccel(ev("F10", { altKey: true, shiftKey: true }))).toBe("Alt+Shift+F10");
    expect(eventToAccel(ev("Digit1", { metaKey: true }))).toBe("Win+1");
  });
  it("waits while only modifiers are held", () => {
    expect(eventToAccel(ev("ControlLeft", { ctrlKey: true }))).toBeNull();
    expect(eventToAccel(ev("ShiftRight", { shiftKey: true }))).toBeNull();
  });
  it("rejects bare typing keys but allows function keys", () => {
    expect(isAcceptable("Z")).toBe(false);
    expect(isAcceptable("Alt+Z")).toBe(true);
    expect(isAcceptable("F9")).toBe(true);
  });
  it("splits accelerators for display", () => {
    expect(accelParts("Ctrl+Alt+M")).toEqual(["Ctrl", "Alt", "M"]);
    expect(accelParts("")).toEqual([]);
  });
});

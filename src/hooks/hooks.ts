import { useEffect, useRef, useState } from "react";
import { convertFileSrc } from "@tauri-apps/api/core";
import { ipc, on } from "@/lib/ipc";
import type { StatsPayload } from "@/lib/types";

/** Subscribes to a backend event for the lifetime of the component. */
export function useTauriEvent<P>(event: string, handler: (p: P) => void) {
  const ref = useRef(handler);
  ref.current = handler;
  useEffect(() => {
    const p = on<P>(event, (x) => ref.current(x));
    return () => void p.then((u) => u());
  }, [event]);
}

/** Tracks whether the window is on screen. The `data-hidden` attribute is maintained globally in `boot()`. */
export function useVisible(): boolean {
  const [v, setV] = useState(!document.hidden);
  useEffect(() => {
    const h = () => setV(!document.hidden);
    document.addEventListener("visibilitychange", h);
    return () => document.removeEventListener("visibilitychange", h);
  }, []);
  return v;
}

/** Live stats stream. The backend only samples while at least one subscriber is active. */
export function useStats(active: boolean): StatsPayload | null {
  const [stats, setStats] = useState<StatsPayload | null>(null);
  useEffect(() => {
    if (!active) return;
    void ipc.statsSubscribe(true);
    const p = on<StatsPayload>("stats", setStats);
    return () => {
      void ipc.statsSubscribe(false);
      void p.then((u) => u());
    };
  }, [active]);
  return active ? stats : null;
}

/** Microphone / system peak levels (0..1), streamed at ~15 Hz while `active`. */
export function useLevels(active: boolean): { mic: number; system: number } {
  const [lv, setLv] = useState({ mic: 0, system: 0 });
  useEffect(() => {
    if (!active) {
      setLv({ mic: 0, system: 0 });
      return;
    }
    void ipc.levelsSubscribe(true);
    const p = on<{ mic: number; system: number }>("audio-levels", setLv);
    return () => {
      void ipc.levelsSubscribe(false);
      void p.then((u) => u());
    };
  }, [active]);
  return lv;
}

/** Ticks `now` every `ms` while active – used for the recording timer. */
export function useNow(active: boolean, ms = 250): number {
  const [now, setNow] = useState(Date.now());
  useEffect(() => {
    if (!active) return;
    setNow(Date.now());
    const id = setInterval(() => setNow(Date.now()), ms);
    return () => clearInterval(id);
  }, [active, ms]);
  return now;
}

export const assetUrl = (path: string) => (path ? convertFileSrc(path) : "");

export function useDebounced<T>(value: T, ms = 250): T {
  const [v, setV] = useState(value);
  useEffect(() => {
    const id = setTimeout(() => setV(value), ms);
    return () => clearTimeout(id);
  }, [value, ms]);
  return v;
}

/** Closes on Escape / outside click. */
export function useDismiss(open: boolean, close: () => void, ref: React.RefObject<HTMLElement | null>) {
  useEffect(() => {
    if (!open) return;
    const key = (e: KeyboardEvent) => e.key === "Escape" && close();
    const down = (e: MouseEvent) => ref.current && !ref.current.contains(e.target as Node) && close();
    window.addEventListener("keydown", key);
    window.addEventListener("mousedown", down);
    return () => {
      window.removeEventListener("keydown", key);
      window.removeEventListener("mousedown", down);
    };
  }, [open, close, ref]);
}

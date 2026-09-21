import { useEffect } from "react";

type Dir = "left" | "right" | "up" | "down";

/** Pure spatial search: the nearest rect in a direction, favouring alignment on the other axis. */
export function nextInDirection(from: DOMRect, candidates: DOMRect[], dir: Dir): number {
  const c = (r: DOMRect) => ({ x: r.left + r.width / 2, y: r.top + r.height / 2 });
  const f = c(from);
  let best = -1;
  let bestScore = Infinity;
  candidates.forEach((r, i) => {
    const p = c(r);
    const dx = p.x - f.x;
    const dy = p.y - f.y;
    const primary = dir === "right" ? dx : dir === "left" ? -dx : dir === "down" ? dy : -dy;
    if (primary <= 4) return;
    const secondary = dir === "left" || dir === "right" ? Math.abs(dy) : Math.abs(dx);
    const score = primary + secondary * 2.4;
    if (score < bestScore) { bestScore = score; best = i; }
  });
  return best;
}

export function moveFocus(root: HTMLElement, dir: Dir) {
  const items = Array.from(root.querySelectorAll<HTMLElement>("[data-nav]:not([disabled])")).filter((e) => e.offsetParent !== null);
  if (items.length === 0) return;
  const cur = document.activeElement as HTMLElement | null;
  const i = cur ? items.indexOf(cur) : -1;
  if (i < 0) { items[0].focus(); return; }
  const j = nextInDirection(items[i].getBoundingClientRect(), items.map((e) => e.getBoundingClientRect()), dir);
  if (j >= 0) items[j].focus();
}

/** Arrow-key spatial navigation inside `root`, active while `enabled`. */
export function useSpatialNav(root: React.RefObject<HTMLElement | null>, enabled: boolean) {
  useEffect(() => {
    if (!enabled) return;
    const k = (e: KeyboardEvent) => {
      const el = root.current;
      if (!el) return;
      const tag = (e.target as HTMLElement)?.tagName;
      if (tag === "INPUT" && (e.target as HTMLInputElement).type === "range" && (e.key === "ArrowLeft" || e.key === "ArrowRight")) return; // let sliders adjust
      const map: Record<string, Dir> = { ArrowLeft: "left", ArrowRight: "right", ArrowUp: "up", ArrowDown: "down" };
      const d = map[e.key];
      if (d) { e.preventDefault(); moveFocus(el, d); }
    };
    window.addEventListener("keydown", k);
    return () => window.removeEventListener("keydown", k);
  }, [root, enabled]);
}

/**
 * Controller support through the Gamepad API: d-pad / left stick move focus, A activates, B closes.
 * The polling loop only runs while `enabled` (overlay visible) and a controller is connected.
 */
export function useGamepadNav(root: React.RefObject<HTMLElement | null>, enabled: boolean, onBack: () => void) {
  useEffect(() => {
    if (!enabled) return;
    let raf = 0;
    const held: Record<string, number> = {};
    const step = (now: number) => {
      const pad = Array.from(navigator.getGamepads?.() ?? []).find((p) => p && p.connected);
      if (pad) {
        const pressed = (i: number) => !!pad.buttons[i]?.pressed;
        const ax = pad.axes[0] ?? 0, ay = pad.axes[1] ?? 0;
        const inputs: Record<string, boolean> = {
          left: pressed(14) || ax < -0.6, right: pressed(15) || ax > 0.6, up: pressed(12) || ay < -0.6, down: pressed(13) || ay > 0.6, a: pressed(0), b: pressed(1),
        };
        for (const [k, on] of Object.entries(inputs)) {
          if (!on) { delete held[k]; continue; }
          const first = held[k] === undefined;
          if (first || now - held[k] > 260) {
            held[k] = first ? now + 200 : now;
            if (k === "a") (document.activeElement as HTMLElement | null)?.click();
            else if (k === "b") onBack();
            else if (root.current) moveFocus(root.current, k as Dir);
          }
        }
      }
      raf = requestAnimationFrame(step);
    };
    raf = requestAnimationFrame(step);
    return () => cancelAnimationFrame(raf);
  }, [root, enabled, onBack]);
}

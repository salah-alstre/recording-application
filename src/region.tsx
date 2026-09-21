import { useEffect, useRef, useState } from "react";
import { createRoot } from "react-dom/client";
import { Check, X } from "lucide-react";
import { boot } from "./boot";
import { useT } from "@/i18n";
import { ipc, on } from "@/lib/ipc";
import type { Rect } from "@/lib/types";

/** Full-screen drag-to-select. Coordinates are sent in physical pixels relative to the display. */
function Picker() {
  const t = useT();
  const [display, setDisplay] = useState(1);
  const [drag, setDrag] = useState<{ x: number; y: number } | null>(null);
  const [rect, setRect] = useState<{ x: number; y: number; w: number; h: number } | null>(null);
  const root = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const p = on<{ displayIndex: number }>("region-open", (e) => { setDisplay(e.displayIndex); setRect(null); setDrag(null); root.current?.focus(); });
    const k = (e: KeyboardEvent) => { if (e.key === "Escape") void ipc.regionCancel(); if (e.key === "Enter") confirm(); };
    window.addEventListener("keydown", k);
    return () => { void p.then((u) => u()); window.removeEventListener("keydown", k); };
  });

  const confirm = () => {
    if (!rect || rect.w < 16 || rect.h < 16) return;
    const dpr = window.devicePixelRatio || 1;
    const r: Rect = { x: Math.round(rect.x * dpr), y: Math.round(rect.y * dpr), w: Math.round(rect.w * dpr) & ~1, h: Math.round(rect.h * dpr) & ~1 };
    setRect(null);
    void ipc.regionConfirm(display, r);
  };

  const down = (e: React.PointerEvent) => { (e.target as HTMLElement).setPointerCapture?.(e.pointerId); setDrag({ x: e.clientX, y: e.clientY }); setRect({ x: e.clientX, y: e.clientY, w: 0, h: 0 }); };
  const move = (e: React.PointerEvent) => { if (!drag) return; setRect({ x: Math.min(drag.x, e.clientX), y: Math.min(drag.y, e.clientY), w: Math.abs(e.clientX - drag.x), h: Math.abs(e.clientY - drag.y) }); };
  const up = () => setDrag(null);
  const dpr = window.devicePixelRatio || 1;

  return (
    <div ref={root} tabIndex={-1} className="region-root" onPointerDown={down} onPointerMove={move} onPointerUp={up}>
      {!rect && <div className="region-hint">{t("region.hint")}</div>}
      {rect && (
        <>
          <div className="region-box" style={{ left: rect.x, top: rect.y, width: rect.w, height: rect.h }}>
            <span className="region-size mono num">{Math.round(rect.w * dpr)} × {Math.round(rect.h * dpr)}</span>
          </div>
          {!drag && rect.w >= 16 && rect.h >= 16 && (
            <div className="region-actions" style={{ left: rect.x + rect.w - 8, top: rect.y + rect.h + 10 }} onPointerDown={(e) => e.stopPropagation()}>
              <button className="btn sm" onClick={() => void ipc.regionCancel()}><X size={14} />{t("common.cancel")}</button>
              <button className="btn sm primary" onClick={confirm}><Check size={14} />{t("common.confirm")}</button>
            </div>
          )}
        </>
      )}
    </div>
  );
}

void boot({ transparent: true }).then(() => createRoot(document.getElementById("root")!).render(<Picker />));

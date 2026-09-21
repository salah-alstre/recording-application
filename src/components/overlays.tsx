import { useEffect, useRef, useState, type ReactNode } from "react";
import { createPortal } from "react-dom";
import { AlertTriangle, Info, X } from "lucide-react";
import { useT } from "@/i18n";
import { describeError, useLang } from "@/i18n";
import type { AppError } from "@/lib/types";

export function Dialog({ open, onClose, title, children, actions, wide }: { open: boolean; onClose: () => void; title: string; children: ReactNode; actions?: ReactNode; wide?: boolean }) {
  const ref = useRef<HTMLDivElement>(null);
  useEffect(() => {
    if (!open) return;
    const prev = document.activeElement as HTMLElement | null;
    ref.current?.focus();
    const k = (e: KeyboardEvent) => e.key === "Escape" && onClose();
    window.addEventListener("keydown", k);
    return () => { window.removeEventListener("keydown", k); prev?.focus?.(); };
  }, [open, onClose]);
  if (!open) return null;
  return createPortal(
    <div className="dialog-backdrop" onMouseDown={(e) => e.target === e.currentTarget && onClose()}>
      <div ref={ref} tabIndex={-1} role="dialog" aria-modal="true" aria-label={title} className={`dialog pop-in${wide ? " wide" : ""}`}>
        <h2>{title}</h2>
        {children}
        {actions && <div className="dialog-actions">{actions}</div>}
      </div>
    </div>, document.body);
}

export interface MenuEntry { label: string; icon?: ReactNode; onClick: () => void; danger?: boolean; separatorBefore?: boolean }

export function ContextMenu({ at, entries, onClose }: { at: { x: number; y: number } | null; entries: MenuEntry[]; onClose: () => void }) {
  const ref = useRef<HTMLDivElement>(null);
  const [pos, setPos] = useState({ left: 0, top: 0 });
  useEffect(() => {
    if (!at) return;
    const z = Number(getComputedStyle(document.documentElement).getPropertyValue("--ui-scale")) || 1;
    const w = 220, h = entries.length * 36 + 16;
    setPos({ left: Math.min(at.x, window.innerWidth - w * z - 8) / z, top: Math.min(at.y, window.innerHeight - h * z - 8) / z });
    ref.current?.querySelector<HTMLElement>("button")?.focus();
    const down = (e: MouseEvent) => ref.current && !ref.current.contains(e.target as Node) && onClose();
    const key = (e: KeyboardEvent) => {
      if (e.key === "Escape") onClose();
      const items = Array.from(ref.current?.querySelectorAll<HTMLElement>("button") ?? []);
      const i = items.indexOf(document.activeElement as HTMLElement);
      if (e.key === "ArrowDown") { e.preventDefault(); items[(i + 1) % items.length]?.focus(); }
      if (e.key === "ArrowUp") { e.preventDefault(); items[(i - 1 + items.length) % items.length]?.focus(); }
    };
    window.addEventListener("mousedown", down);
    window.addEventListener("keydown", key);
    return () => { window.removeEventListener("mousedown", down); window.removeEventListener("keydown", key); };
  }, [at, entries.length, onClose]);
  if (!at) return null;
  return createPortal(
    <div ref={ref} className="popover pop-in" role="menu" style={{ left: pos.left, top: pos.top, width: 220 }}>
      {entries.map((e, i) => (
        <div key={i}>
          {e.separatorBefore && <div className="menu-sep" />}
          <button role="menuitem" className={`menu-item${e.danger ? " danger" : ""}`} onClick={() => { onClose(); e.onClick(); }}>
            {e.icon}<span>{e.label}</span>
          </button>
        </div>
      ))}
    </div>, document.body);
}

export function ErrorBanner({ error, onDismiss, action }: { error: AppError; onDismiss?: () => void; action?: ReactNode }) {
  const lang = useLang((s) => s.lang);
  const { title, hint } = describeError(lang, error);
  const t = useT();
  return (
    <div className="banner error pop-in" role="alert">
      <AlertTriangle size={18} style={{ color: "var(--danger)", flex: "none", marginTop: 2 }} />
      <div style={{ flex: 1, minWidth: 0 }}>
        <div className="b-title">{title}</div>
        {hint && <div className="b-body">{hint}</div>}
      </div>
      {action}
      {onDismiss && <button className="btn ghost icon sm" aria-label={t("common.dismiss")} onClick={onDismiss}><X size={15} /></button>}
    </div>
  );
}

export function InfoBanner({ children, tone = "info" }: { children: ReactNode; tone?: "info" | "warn" }) {
  return (
    <div className={`banner${tone === "warn" ? " warn" : ""}`}>
      <Info size={17} style={{ flex: "none", marginTop: 2, color: tone === "warn" ? "var(--warn)" : "var(--accent-strong)" }} />
      <div className="b-body" style={{ flex: 1 }}>{children}</div>
    </div>
  );
}

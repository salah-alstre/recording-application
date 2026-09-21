import type { CSSProperties, ReactNode } from "react";

export function Skeleton({ w, h, r, style }: { w?: number | string; h?: number | string; r?: number; style?: CSSProperties }) {
  return <div className="skeleton" aria-hidden style={{ width: w, height: h, borderRadius: r, ...style }} />;
}

export function Meter({ level, off, label }: { level: number; off?: boolean; label: string }) {
  // perceptual curve so quiet speech is visible
  const lv = Math.min(100, Math.round(Math.pow(Math.min(1, level), 0.55) * 100));
  return (
    <div className={`meter${off ? " off" : ""}`} role="meter" aria-label={label} aria-valuemin={0} aria-valuemax={100} aria-valuenow={off ? 0 : lv}>
      <i style={{ ["--lv" as string]: off ? "0%" : `${lv}%` }} />
    </div>
  );
}

export function EmptyState({ icon, title, body, action }: { icon: ReactNode; title: string; body?: string; action?: ReactNode }) {
  return (
    <div className="empty">
      <div className="empty-art">{icon}</div>
      <h3>{title}</h3>
      {body && <p>{body}</p>}
      {action}
    </div>
  );
}

export function Chip({ children, tone, icon }: { children: ReactNode; tone?: "accent" | "ok" | "warn" | "danger"; icon?: ReactNode }) {
  return <span className={`chip${tone ? " " + tone : ""}`}>{icon}{children}</span>;
}

export function Progress({ value, indeterminate }: { value?: number; indeterminate?: boolean }) {
  return (
    <div className={`progress${indeterminate ? " indeterminate" : ""}`} role="progressbar" aria-valuenow={indeterminate ? undefined : Math.round(value ?? 0)} aria-valuemin={0} aria-valuemax={100}>
      <i style={indeterminate ? undefined : { width: `${value ?? 0}%` }} />
    </div>
  );
}

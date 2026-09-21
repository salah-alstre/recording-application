import { useCallback, useEffect, useRef, useState, type ReactNode } from "react";
import { createPortal } from "react-dom";
import { Check, ChevronDown } from "lucide-react";
import { useDismiss } from "@/hooks/hooks";
import { accelParts, KEY_LABELS } from "@/lib/hotkey";

export function Toggle({ checked, onChange, label, disabled }: { checked: boolean; onChange: (v: boolean) => void; label: string; disabled?: boolean }) {
  return (
    <button type="button" role="switch" aria-checked={checked} aria-label={label} disabled={disabled} className="toggle" onClick={() => onChange(!checked)} />
  );
}

export function Slider({ value, min, max, step = 1, onChange, label, format }: { value: number; min: number; max: number; step?: number; onChange: (v: number) => void; label: string; format?: (v: number) => string }) {
  const p = ((value - min) / (max - min)) * 100;
  return (
    <>
      <input type="range" className="slider" aria-label={label} min={min} max={max} step={step} value={value} style={{ ["--p" as string]: `${p}%` }} onChange={(e) => onChange(Number(e.target.value))} />
      <span className="slider-val">{format ? format(value) : value}</span>
    </>
  );
}

export interface Option<T extends string | number> { value: T; label: string; icon?: ReactNode; hint?: string }

export function Segmented<T extends string | number>({ value, options, onChange, label }: { value: T; options: Option<T>[]; onChange: (v: T) => void; label: string }) {
  return (
    <div className="seg" role="group" aria-label={label}>
      {options.map((o) => (
        <button key={String(o.value)} type="button" aria-pressed={o.value === value} onClick={() => onChange(o.value)} title={o.hint}>
          {o.value === value && <span className="seg-pill" />}
          {o.icon}
          <span>{o.label}</span>
        </button>
      ))}
    </div>
  );
}

export function Select<T extends string | number>({ value, options, onChange, label, placeholder, disabled }: { value: T; options: Option<T>[]; onChange: (v: T) => void; label: string; placeholder?: string; disabled?: boolean }) {
  const [open, setOpen] = useState(false);
  const [active, setActive] = useState(0);
  const btn = useRef<HTMLButtonElement>(null);
  const pop = useRef<HTMLDivElement>(null);
  const [pos, setPos] = useState({ left: 0, top: 0, width: 200 });
  const close = useCallback(() => setOpen(false), []);
  useDismiss(open, close, pop);
  const current = options.find((o) => o.value === value);

  const show = () => {
    const r = btn.current?.getBoundingClientRect();
    if (!r) return;
    const z = Number(getComputedStyle(document.documentElement).getPropertyValue("--ui-scale")) || 1;
    const below = window.innerHeight - r.bottom > 240;
    setPos({ left: r.left / z, top: (below ? r.bottom + 6 : Math.max(8, r.top - 6 - Math.min(320, options.length * 38 + 12))) / z, width: r.width / z });
    setActive(Math.max(0, options.findIndex((o) => o.value === value)));
    setOpen(true);
  };

  useEffect(() => {
    if (open) pop.current?.focus();
  }, [open]);

  const onKey = (e: React.KeyboardEvent) => {
    if (e.key === "ArrowDown") { e.preventDefault(); setActive((a) => Math.min(options.length - 1, a + 1)); }
    else if (e.key === "ArrowUp") { e.preventDefault(); setActive((a) => Math.max(0, a - 1)); }
    else if (e.key === "Enter" || e.key === " ") { e.preventDefault(); const o = options[active]; if (o) { onChange(o.value); setOpen(false); btn.current?.focus(); } }
    else if (e.key === "Escape") { setOpen(false); btn.current?.focus(); }
  };

  return (
    <>
      <button ref={btn} type="button" className="select-btn" aria-haspopup="listbox" aria-expanded={open} aria-label={label} disabled={disabled} onClick={() => (open ? close() : show())}>
        <span>{current?.label ?? placeholder ?? ""}</span>
        <ChevronDown size={16} className="dim" />
      </button>
      {open && createPortal(
        <div ref={pop} className="popover" role="listbox" tabIndex={-1} aria-label={label} onKeyDown={onKey} style={{ left: pos.left, top: pos.top, minWidth: pos.width }}>
          {options.map((o, i) => (
            <button key={String(o.value)} type="button" role="option" className="opt" aria-selected={o.value === value} data-active={i === active} onMouseEnter={() => setActive(i)} onClick={() => { onChange(o.value); setOpen(false); }}>
              <span style={{ flex: 1 }}>{o.label}</span>
              {o.hint && <span className="faint" style={{ fontSize: 12 }}>{o.hint}</span>}
              {o.value === value && <Check size={15} />}
            </button>
          ))}
        </div>, document.body)}
    </>
  );
}

export function Field({ label, hint, children, wide }: { label: string; hint?: string; children: ReactNode; wide?: boolean }) {
  return (
    <div className="field">
      <div className="field-text">
        <div className="field-label">{label}</div>
        {hint && <div className="field-hint">{hint}</div>}
      </div>
      <div className={`field-control${wide ? " wide" : ""}`}>{children}</div>
    </div>
  );
}

export function KeyCaps({ accel }: { accel: string }) {
  const parts = accelParts(accel);
  return (
    <span className="keys" aria-label={accel}>
      {parts.map((p, i) => <kbd key={i} className="keycap">{KEY_LABELS[p] ?? p}</kbd>)}
    </span>
  );
}

export function NumberInput({ value, onChange, min, max, label, width = 96 }: { value: number; onChange: (v: number) => void; min?: number; max?: number; label: string; width?: number }) {
  return (
    <input className="input mono" type="number" aria-label={label} style={{ width }} value={value} min={min} max={max}
      onChange={(e) => { const v = Number(e.target.value); if (Number.isFinite(v)) onChange(Math.min(max ?? Infinity, Math.max(min ?? -Infinity, v))); }} />
  );
}

import { useId } from "react";

/** Rimlight mark: a lens ring lit from one edge, with a solid core. */
export function Logo({ size = 28, recording = false }: { size?: number; recording?: boolean }) {
  const id = useId();
  return (
    <svg className="logo-mark" width={size} height={size} viewBox="0 0 64 64" role="img" aria-label="Rimlight">
      <defs>
        <linearGradient id={`${id}-r`} x1="0.1" y1="0.1" x2="0.9" y2="0.9">
          <stop offset="0" stopColor="#ffc496" />
          <stop offset="0.45" stopColor="#8f9bd0" />
          <stop offset="1" stopColor="#3d476b" />
        </linearGradient>
        <linearGradient id={`${id}-bg`} x1="0" y1="0" x2="0" y2="1">
          <stop offset="0" stopColor="#171b2b" />
          <stop offset="1" stopColor="#0a0c14" />
        </linearGradient>
      </defs>
      <rect x="1" y="1" width="62" height="62" rx="18" fill={`url(#${id}-bg)`} />
      <circle cx="32" cy="32" r="16.5" fill="none" stroke={`url(#${id}-r)`} strokeWidth="5.4" />
      <circle cx="32" cy="32" r="6.2" fill={recording ? "#ff5c6c" : "#eceff6"} />
    </svg>
  );
}

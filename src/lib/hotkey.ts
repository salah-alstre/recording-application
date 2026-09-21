const CODE_NAMES: Record<string, string> = {
  Space: "Space", Enter: "Enter", Tab: "Tab", Backspace: "Backspace", Escape: "Escape", Insert: "Insert", Delete: "Delete",
  Home: "Home", End: "End", PageUp: "PageUp", PageDown: "PageDown",
  ArrowUp: "ArrowUp", ArrowDown: "ArrowDown", ArrowLeft: "ArrowLeft", ArrowRight: "ArrowRight",
  Comma: "Comma", Period: "Period", Slash: "Slash", Semicolon: "Semicolon", Quote: "Quote", BracketLeft: "BracketLeft",
  BracketRight: "BracketRight", Backslash: "Backslash", Minus: "Minus", Equal: "Equal", Backquote: "Backquote",
  PrintScreen: "PrintScreen", Pause: "Pause", ScrollLock: "ScrollLock",
};

const MODIFIER_CODES = /^(Control|Alt|Shift|Meta|OS)(Left|Right)?$/;

/** Converts a keydown into a global-shortcut string like `Ctrl+Alt+M`. Returns null while only modifiers are held. */
export function eventToAccel(e: Pick<KeyboardEvent, "code" | "ctrlKey" | "altKey" | "shiftKey" | "metaKey">): string | null {
  if (MODIFIER_CODES.test(e.code)) return null;
  let key: string | undefined;
  if (/^Key[A-Z]$/.test(e.code)) key = e.code.slice(3);
  else if (/^Digit[0-9]$/.test(e.code)) key = e.code.slice(5);
  else if (/^F([1-9]|1[0-9]|2[0-4])$/.test(e.code)) key = e.code;
  else if (/^Numpad[0-9]$/.test(e.code)) key = "Num" + e.code.slice(6);
  else key = CODE_NAMES[e.code];
  if (!key) return null;
  const mods: string[] = [];
  if (e.ctrlKey) mods.push("Ctrl");
  if (e.altKey) mods.push("Alt");
  if (e.shiftKey) mods.push("Shift");
  if (e.metaKey) mods.push("Win");
  return [...mods, key].join("+");
}

/** A hotkey must include a modifier unless it is a function key (bare letters would break typing everywhere). */
export function isAcceptable(accel: string): boolean {
  const parts = accel.split("+");
  const key = parts[parts.length - 1];
  return parts.length > 1 || /^F([1-9]|1[0-9]|2[0-4])$/.test(key);
}

export function accelParts(accel: string): string[] {
  return accel ? accel.split("+").map((p) => p.trim()).filter(Boolean) : [];
}

export const KEY_LABELS: Record<string, string> = {
  ArrowUp: "↑", ArrowDown: "↓", ArrowLeft: "←", ArrowRight: "→", Backquote: "`", Comma: ",", Period: ".", Slash: "/",
  Semicolon: ";", Quote: "'", BracketLeft: "[", BracketRight: "]", Backslash: "\\", Minus: "-", Equal: "=", Escape: "Esc",
};

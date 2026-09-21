import { create } from "zustand";
import type { AppError, Lang } from "@/lib/types";
import { en } from "./en";
import { ar } from "./ar";

export type Key = keyof typeof en;
export type Params = Record<string, string | number | undefined | null>;

const dicts: Record<Lang, Record<Key, string>> = { en, ar };
const STORAGE_KEY = "rimlight.lang";

function initialLang(): Lang {
  try {
    const v = localStorage.getItem(STORAGE_KEY);
    if (v === "ar" || v === "en") return v;
  } catch { /* storage unavailable */ }
  return typeof navigator !== "undefined" && navigator.language?.toLowerCase().startsWith("ar") ? "ar" : "en";
}

interface LangState { lang: Lang; setLang: (l: Lang) => void }
export const useLang = create<LangState>((set) => ({
  lang: initialLang(),
  setLang: (lang) => {
    try { localStorage.setItem(STORAGE_KEY, lang); } catch { /* ignore */ }
    applyLangToDocument(lang);
    set({ lang });
  },
}));

export function applyLangToDocument(lang: Lang) {
  const root = document.documentElement;
  root.lang = lang;
  root.dir = lang === "ar" ? "rtl" : "ltr";
}

export function interpolate(template: string, params?: Params): string {
  if (!params) return template;
  return template.replace(/\{(\w+)\}/g, (_, k: string) => (params[k] === undefined || params[k] === null ? "" : String(params[k])));
}

export function translate(lang: Lang, key: Key, params?: Params): string {
  return interpolate(dicts[lang][key] ?? dicts.en[key] ?? key, params);
}

export function t(key: Key, params?: Params): string {
  return translate(useLang.getState().lang, key, params);
}

export type TFn = (key: Key, params?: Params) => string;

/** Reactive translator: components re-render when the language changes. */
export function useT(): TFn {
  const lang = useLang((s) => s.lang);
  return (key, params) => translate(lang, key, params);
}

export function has(lang: Lang, key: string): key is Key {
  return key in dicts[lang];
}

/** Localised title/hint for a backend error. Unknown codes fall back to the backend's own English text. */
export function describeError(lang: Lang, err: AppError): { title: string; hint: string } {
  const k = `err.${err.code}`;
  const title = has(lang, k) ? translate(lang, k) : err.message;
  const hk = `hint.${err.code}`;
  const hint = has(lang, hk) ? translate(lang, hk) : err.hint ?? "";
  return { title, hint };
}

/** Numbers inside RTL text keep Latin digits (tech context) but must stay left-to-right. */
export const isRtl = (lang: Lang) => lang === "ar";

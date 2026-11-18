import { ru, type Forms } from './ru.ts';

/**
 * The app's words in the reader's language (ADR 0114). The English text is the key:
 * `t('Home')` is "Home" in English and the catalogue's entry in Russian; anything missing
 * stays English. `{name}` slots are filled from `vars`. People's own words are never passed
 * through here.
 */
export type Language = 'en' | 'ru';

export const LANGUAGES: { value: Language; label: string }[] = [
  { value: 'en', label: 'English' },
  { value: 'ru', label: 'Русский' },
];

const KEY = 'sp_language';

function stored(): Language | null {
  try {
    const v = localStorage.getItem(KEY);
    return v === 'en' || v === 'ru' ? v : null;
  } catch {
    return null;
  }
}

/**
 * The first time, the browser's (or the device's) main language decides (owner, 2026-11-18):
 * only the first one, so a phone set to English that also lists Russian stays English.
 */
function detect(): Language {
  const s = stored();
  if (s) return s;
  const main =
    typeof navigator === 'undefined' ? '' : (navigator.languages?.[0] ?? navigator.language);
  return main?.toLowerCase().startsWith('ru') ? 'ru' : 'en';
}

export const language: Language = detect();

if (typeof document !== 'undefined') document.documentElement.lang = language;

/** Switch language: kept on this device, and the app reloads in it. */
export function setLanguage(next: Language): void {
  try {
    localStorage.setItem(KEY, next);
  } catch {
    // Storage refused: the reload below still shows the choice until the app closes.
  }
  location.reload();
}

/** The locale for dates and numbers. */
export const locale = (): string => (language === 'ru' ? 'ru-RU' : 'en-AU');

const fill = (text: string, vars?: Record<string, string | number>): string =>
  vars ? text.replace(/\{(\w+)\}/g, (m, k: string) => (k in vars ? String(vars[k]) : m)) : text;

export function t(text: string, vars?: Record<string, string | number>): string {
  if (language === 'ru') {
    const hit = ru[text];
    if (typeof hit === 'string') return fill(hit, vars);
  }
  return fill(text, vars);
}

const rules = new Intl.PluralRules('ru-RU');

/**
 * A count and its noun: `plural(n, '{n} member', '{n} members')`. Russian keys on the
 * English plural and has one, few and many forms.
 */
export function plural(
  n: number,
  one: string,
  other: string,
  vars?: Record<string, string | number>,
): string {
  const all = { n: n.toLocaleString(locale()), ...vars };
  if (language === 'ru') {
    const hit = ru[other];
    if (hit && typeof hit === 'object') {
      const form = rules.select(n) as keyof Forms;
      return fill(hit[form] ?? hit.many, all);
    }
  }
  return fill(n === 1 ? one : other, all);
}

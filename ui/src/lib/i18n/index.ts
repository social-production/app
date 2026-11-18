import { LANGUAGES, type LanguageInfo } from './languages.ts';

export { LANGUAGES, type LanguageInfo } from './languages.ts';

/**
 * The app's words in the reader's language (ADR 0114). The English text is the key:
 * `t('Home')` is "Home" in English and the catalogue's entry in another language; anything
 * missing stays English. `{name}` slots are filled from `vars`. People's own words are never
 * passed through here.
 */
export type Language = string;

const KEY = 'sp_language';
const known = (code: string | null | undefined): LanguageInfo | undefined =>
  LANGUAGES.find((l) => l.value === code);

function stored(): string | null {
  try {
    return known(localStorage.getItem(KEY))?.value ?? null;
  } catch {
    return null;
  }
}

/**
 * The first time, the browser's (or the device's) main language decides (owner, 2026-11-18):
 * only the first one, so a phone set to English that also lists Russian stays English.
 */
function detect(): string {
  const s = stored();
  if (s) return s;
  const main =
    typeof navigator === 'undefined' ? '' : (navigator.languages?.[0] ?? navigator.language);
  return known(main?.toLowerCase().split('-')[0])?.value ?? 'en';
}

export const language: Language = detect();
const current: LanguageInfo = known(language) ?? (LANGUAGES[0] as LanguageInfo);

if (typeof document !== 'undefined') document.documentElement.lang = language;

/** Switch language: kept on this device, and the app reloads in it. */
export function setLanguage(next: Language): void {
  if (next === language || !known(next)) return;
  try {
    localStorage.setItem(KEY, next);
  } catch {
    // Storage refused: the reload below still shows the choice until the app closes.
  }
  location.reload();
}

/** The locale for dates and numbers. */
export const locale = (): string => current.locale;

const fill = (text: string, vars?: Record<string, string | number>): string =>
  vars ? text.replace(/\{(\w+)\}/g, (m, k: string) => (k in vars ? String(vars[k]) : m)) : text;

export function t(text: string, vars?: Record<string, string | number>): string {
  const hit = current.catalogue?.[text];
  return fill(typeof hit === 'string' ? hit : text, vars);
}

const rules = new Intl.PluralRules(current.locale);

/**
 * A count and its noun: `plural(n, '{n} member', '{n} members')`. Other languages key on the
 * English plural, with a form for each of their plural categories (Russian: one, few, many).
 */
export function plural(
  n: number,
  one: string,
  other: string,
  vars?: Record<string, string | number>,
): string {
  const all = { n: n.toLocaleString(current.locale), ...vars };
  const hit = current.catalogue?.[other];
  if (hit && typeof hit === 'object') {
    const form = hit[rules.select(n)] ?? hit.other ?? hit.many;
    if (form) return fill(form, all);
  }
  return fill(n === 1 ? one : other, all);
}

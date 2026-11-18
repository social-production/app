import { ru } from './ru.ts';

/**
 * The languages the app speaks (ADR 0114). To add one: a catalogue like `ru.ts` (English text
 * → that language), then a line here. The picker, the first-open guess and dates follow.
 */

/** A plural entry: Intl.PluralRules' categories for the language (Russian: one, few, many). */
export type Forms = Partial<Record<Intl.LDMLPluralRule, string>>;
export type Entry = string | Forms;

export interface LanguageInfo {
  /** BCP 47 code; also what the device's language is matched on (its first part). */
  value: string;
  /** The language's own name, so people can find theirs whatever is showing. */
  label: string;
  /** For dates and numbers. */
  locale: string;
  /** English text → this language; English has none. */
  catalogue: Record<string, Entry> | null;
}

export const LANGUAGES: LanguageInfo[] = [
  { value: 'en', label: 'English', locale: 'en-AU', catalogue: null },
  { value: 'ru', label: 'Русский', locale: 'ru-RU', catalogue: ru },
];

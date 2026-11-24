import { getContext, setContext } from 'svelte';

import type { CharterMode } from '#lib/domain/projects.ts';

/** What a quote post's card shows of the quoted space (Owner Notes Mockups §6). The shell
 * provides it (CODE_RULES: components don't read services). */
export interface QuotedSpace {
  title: string;
  mode: CharterMode;
  tag: string | null;
}

export interface QuoteSource {
  summary: (id: string) => Promise<QuotedSpace | null>;
}

const KEY = Symbol('quote-source');

export function provideQuoteSource(source: QuoteSource): void {
  setContext(KEY, source);
}

export function quoteSource(): QuoteSource | undefined {
  return getContext<QuoteSource | undefined>(KEY);
}

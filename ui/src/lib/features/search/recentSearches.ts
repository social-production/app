/**
 * Recent searches, kept only in this browser (owner, 2026-11-16: never on the network).
 * Storage can be missing or refused (private windows), so every read and write is guarded.
 */
const KEY = 'sp-recent-searches';
const MAX = 8;

export function readRecent(): string[] {
  try {
    const value: unknown = JSON.parse(localStorage.getItem(KEY) ?? '[]');
    return Array.isArray(value) ? value.filter((v): v is string => typeof v === 'string') : [];
  } catch {
    return [];
  }
}

function write(list: string[]): string[] {
  try {
    localStorage.setItem(KEY, JSON.stringify(list));
  } catch {
    // Not kept; the list still updates for this visit.
  }
  return list;
}

/** Puts `query` first, without repeats (ignoring case). */
export function rememberSearch(query: string): string[] {
  const q = query.trim();
  if (!q) return readRecent();
  const rest = readRecent().filter((r) => r.toLowerCase() !== q.toLowerCase());
  return write([q, ...rest].slice(0, MAX));
}

export function forgetSearch(query: string): string[] {
  return write(readRecent().filter((r) => r !== query));
}

export function clearSearches(): string[] {
  return write([]);
}

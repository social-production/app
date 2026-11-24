/**
 * Quote posts (Owner Notes Mockups §6): a post whose last line is a link to something in the
 * app shows that thing as a quoted card under its words, like a quoted tweet.
 */

const SPACE = /^\/(projects|events|help-requests|threads|posts)\/([^/?#\s]+)\/?(?:[?#].*)?$/;

export interface Quote {
  /** The post's words without the link. */
  text: string;
  id: string;
  href: string;
}

/** The quoted space when the post ends with a link to one (on this site, or a bare path). */
export function quoteOf(body: string, origin: string): Quote | null {
  const trimmed = body.trimEnd();
  const lastLine = trimmed.slice(trimmed.lastIndexOf('\n') + 1).trim();
  const last = lastLine.split(/\s+/).pop() ?? '';
  let path = last;
  if (/^https?:\/\//.test(last)) {
    try {
      const url = new URL(last);
      if (url.origin !== origin) return null;
      path = `${url.pathname}${url.search}`;
    } catch {
      return null;
    }
  }
  const match = SPACE.exec(path);
  if (!match?.[2]) return null;
  const text = trimmed.slice(0, trimmed.length - last.length).trimEnd();
  return { text, id: decodeURIComponent(match[2]), href: path };
}

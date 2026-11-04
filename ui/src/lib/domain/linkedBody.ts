/**
 * web's LinkedPostBody segments. web-backend sends no `linkedSubjects`, so web always links
 * what `autoLinksFromText` finds: full URLs and the app's own paths (`/projects/…`,
 * `/events/…`, `/threads/…`, `/messages…`, `/posts/…`, `/profile/…`), each linking to itself.
 */
export type LinkedSegment =
  { kind: 'text'; value: string } | { kind: 'link'; value: string; href: string };

const AUTO_LINK =
  /(https?:\/\/[^\s<]+|\/(?:projects|events|threads|messages|posts|profile)(?:\/[^\s<]+)?(?:\?[^\s<]+)?)/g;

export function linkedSegments(text: string): LinkedSegment[] {
  const labels = [...new Set([...text.matchAll(AUTO_LINK)].map((m) => m[0]))]
    .filter((label) => label.trim().length > 0)
    .sort((a, b) => b.length - a.length);
  if (labels.length === 0) return [{ kind: 'text', value: text }];

  // web's buildSegments: the earliest label from the cursor, the longer one on a tie.
  const lower = text.toLowerCase();
  const segments: LinkedSegment[] = [];
  let cursor = 0;
  while (cursor < text.length) {
    let next: { index: number; label: string } | null = null;
    for (const label of labels) {
      const index = lower.indexOf(label.toLowerCase(), cursor);
      if (index === -1) continue;
      if (
        !next ||
        index < next.index ||
        (index === next.index && label.length > next.label.length)
      ) {
        next = { index, label };
      }
    }
    if (!next) {
      segments.push({ kind: 'text', value: text.slice(cursor) });
      break;
    }
    if (next.index > cursor) segments.push({ kind: 'text', value: text.slice(cursor, next.index) });
    const value = text.slice(next.index, next.index + next.label.length);
    segments.push({ kind: 'link', value, href: next.label });
    cursor = next.index + next.label.length;
  }
  return segments.length > 0 ? segments : [{ kind: 'text', value: text }];
}

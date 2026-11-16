import type { CommentActivity } from './commentNotes.ts';
import type { FeedSort, ProjectCard } from './projects.ts';

/** A profile's tabs (Profiles and Comments mockups §2). */
export type ProfileTab = 'all' | 'posts' | 'comments' | 'projects';

export type ProfileRow =
  { key: string; card: ProjectCard } | { key: string; comment: CommentActivity };

/** Posts and threads are what someone wrote; everything else is a space they started. */
const written = (card: ProjectCard): boolean => card.mode === 'post' || card.mode === 'thread';

const timeOf = (row: ProfileRow): number =>
  'comment' in row ? row.comment.createdAt : row.card.createdAt;

/**
 * The rows a profile tab shows. Cards keep the feed's order; comments join them by time when
 * the order is by time, and follow the cards when it is "Top".
 */
export function profileRows(
  cards: ProjectCard[],
  comments: CommentActivity[],
  tab: ProfileTab,
  sort: FeedSort,
): ProfileRow[] {
  const cardRows: ProfileRow[] = cards
    .filter(
      (c) => tab === 'all' || (tab === 'posts' ? written(c) : tab === 'projects' && !written(c)),
    )
    .map((card) => ({ key: `card:${card.id}`, card }));
  const commentRows: ProfileRow[] =
    tab === 'all' || tab === 'comments'
      ? comments.map((comment) => ({ key: `comment:${comment.id}`, comment }))
      : [];
  if (tab === 'comments') return sort === 'oldest' ? commentRows.reverse() : commentRows;
  if (sort === 'top') return [...cardRows, ...commentRows];
  const rows = [...cardRows, ...commentRows].sort((a, b) => timeOf(b) - timeOf(a));
  return sort === 'oldest' ? rows.reverse() : rows;
}

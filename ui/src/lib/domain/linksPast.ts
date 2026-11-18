/**
 * The Links tab's past list (Polish Pass mockups §6): one row per ended link, saying why it
 * ended, and one per vote that failed, saying who said no. A passed vote isn't listed apart:
 * its link is the row (active above, or ended here).
 */
import { t } from '#lib/i18n/index.ts';
import { shortDate } from './botReview.ts';
import type { LinkRequest, LinkSide, ProjectLink } from './links.ts';

export interface PastRow {
  key: string;
  title: string;
  targetId: string;
  meta: string;
  tag: string;
  /** Shown when the row opens: the link's description, or how its vote went. */
  link: ProjectLink | null;
  request: LinkRequest | null;
  at: number;
}

const day = (micros: number) => shortDate(micros / 1000);

function saidNo(r: LinkRequest): string {
  const no = [r.thisSide, r.otherSide].find((s: LinkSide) => s.half?.status === 'Rejected');
  if (!no) return 'not decided';
  return `${no === r.thisSide ? 'this project' : no.projectTitle} said no`;
}

export function pastRows(links: ProjectLink[], votes: LinkRequest[]): PastRow[] {
  const severs = votes.filter((v) => v.sever && v.status === 'Approved');
  const rows: PastRow[] = links.map((link) => {
    const sever = link.sever ?? severs.find((v) => v.target.id === link.create.target.id) ?? null;
    const at = sever?.createdAt ?? link.create.createdAt;
    return {
      key: link.id,
      title: link.create.target.title,
      targetId: link.create.target.id,
      meta: sever ? `${t('Link ended')} · ${day(at)} · ${t('both sides voted')}` : t('Link ended'),
      tag: 'Ended',
      link,
      request: null,
      at,
    };
  });
  const ended = new Set(links.map((l) => l.create.target.id));
  for (const v of votes) {
    if (v.status === 'Approved' && (!v.sever || ended.has(v.target.id))) continue;
    rows.push({
      key: v.id,
      title: v.target.title,
      targetId: v.target.id,
      meta: `${v.sever ? t('Vote to end the link') : t('Link vote')} · ${day(v.createdAt)} · ${saidNo(v)}`,
      tag: v.sever ? 'Kept' : t('Not linked'),
      link: null,
      request: v,
      at: v.createdAt,
    });
  }
  return rows.sort((a, b) => b.at - a.at);
}

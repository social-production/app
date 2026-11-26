import { callCell, hashToString, projectCell, type AgentPubKey } from '#lib/holo/index.ts';

import { notify } from './notices.ts';
import { cellFor, myProjectSeeds, type ProjectRef } from './projects.ts';

/**
 * Pushes whose recipients this device has to look up first (ADR 0108): a comment's project
 * owner, and the people who share a project with an account under bot review. Best effort,
 * like every notice: the cards are derived on their devices either way.
 */

interface Glance {
  creator: AgentPubKey;
  members: AgentPubKey[];
}

async function glance(cell: Awaited<ReturnType<typeof projectCell>>): Promise<Glance | null> {
  return callCell<Glance | null>(cell, 'project', 'get_project');
}

/** governance.py `_notify_comment_recipients`: the replied-to author, else the project's owner
 * ("Replied to your comment" / "Replied to your project", as the cards say). */
export async function replyNotice(
  project: ProjectRef,
  answered: string | null,
  body: string,
  link: string,
): Promise<void> {
  try {
    if (answered) {
      await notify([answered], 'reply', (me) => `${me} replied to your comment`, body, link);
      return;
    }
    const p = await glance(await cellFor(project));
    if (!p) return;
    const owner = hashToString(p.creator);
    await notify([owner], 'reply', (me) => `${me} replied to your project`, body, link);
  } catch {
    /* Best effort. */
  }
}

/** bot_reviews.py's `bot-review-vote`: a review opened, so the people this device shares a
 * project with the account are asked whether they know them. Markers hear their own card. */
export async function reviewVoteNotice(
  account: string,
  markers: string[],
  copy: [string, string],
): Promise<void> {
  try {
    const seeds = await myProjectSeeds();
    const rosters = await Promise.allSettled(
      seeds.map(async (seed) => {
        const p = await glance(await projectCell(seed, 'Project'));
        return p ? p.members.map(hashToString) : [];
      }),
    );
    const people = new Set<string>();
    for (const r of rosters) {
      if (r.status === 'fulfilled' && r.value.includes(account))
        r.value.forEach((a) => people.add(a));
    }
    for (const a of [account, ...markers]) people.delete(a);
    await notify([...people], 'bot-review-vote', () => copy[0], copy[1], `/profile/${account}`);
  } catch {
    /* Best effort. */
  }
}

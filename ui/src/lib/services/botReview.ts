import type { BotReview } from '#lib/domain/botReview.ts';
import type { BotNotice } from '#lib/domain/botReviewNotifications.ts';
import {
  callZome,
  pausedIndex,
  hashToString,
  stringToHash,
  type ActionHash,
  type AgentPubKey,
} from '#lib/holo/index.ts';

/** web's bot reviews (ADR 0074): marks, review votes and the one re-review. */
const identity = <T>(fn: string, payload?: unknown) =>
  callZome<T>('directory', 'identity', fn, payload);

interface BotReviewWire {
  status: 'none' | 'open' | 'upheld';
  is_appeal: boolean;
  bot_votes: number;
  real_votes: number;
  votes_required: number;
  closes_at: number | null;
  my_vote: boolean | null;
  re_review_from: number | null;
  markers: AgentPubKey[];
  my_cooldown: number | null;
}

const ms = (micros: number | null) => (micros === null ? null : Math.floor(micros / 1000));

export async function getBotReview(agent: string): Promise<BotReview> {
  const w = await identity<BotReviewWire>('get_bot_review', stringToHash(agent));
  return {
    status: w.status,
    isAppeal: w.is_appeal,
    botVotes: w.bot_votes,
    realVotes: w.real_votes,
    votesRequired: w.votes_required,
    closesAt: ms(w.closes_at),
    myVote: w.my_vote === null ? null : w.my_vote ? 'bot' : 'real',
    reReviewFrom: ms(w.re_review_from),
    markers: w.markers.map(hashToString),
    myCooldownUntil: ms(w.my_cooldown),
  };
}

export async function setBotMark(agent: string, bot: boolean): Promise<void> {
  await identity<ActionHash>('set_bot_mark', { account: stringToHash(agent), bot });
}

export async function voteBotReview(agent: string, vote: 'bot' | 'real'): Promise<void> {
  await identity<ActionHash>('vote_bot_review', {
    account: stringToHash(agent),
    bot: vote === 'bot',
  });
}

export async function requestBotReReview(): Promise<void> {
  await identity<ActionHash>('request_bot_re_review');
}

/** Everyone a bot review paused (web's `bootstrap.pausedUsernames`); empty if the node can't say. */
export async function pausedAgents(): Promise<string[]> {
  return pausedIndex().catch(() => []);
}

interface BotNoticeWire {
  account: AgentPubKey;
  kind: BotNotice['kind'];
  is_appeal: boolean;
  at: number;
  re_review_from: number | null;
}

/** Openings and closings of your own review and of those you marked or voted on. */
export async function myBotNotices(): Promise<BotNotice[]> {
  const wire = await identity<BotNoticeWire[]>('my_bot_review_notices');
  return wire.map((n) => ({
    account: hashToString(n.account),
    kind: n.kind,
    isAppeal: n.is_appeal,
    at: n.at,
    reReviewFrom: n.re_review_from,
  }));
}

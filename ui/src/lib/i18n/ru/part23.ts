/** Russian, part of the catalogue (ADR 0114; see ../ru.ts): only Full members vote (ADR 0116). */
import type { Entry } from '../languages.ts';

export const part23: Record<string, Entry> = {
  'Only Full members vote': 'Голосуют только полноправные участники',
  "You're a New member, so you can't vote or start a vote yet. You get Full standing from two vouches by Full members, or after steady activity.":
    'Вы новый участник, поэтому пока не можете голосовать или начинать голосование. Полноправный статус дают два поручительства полноправных участников или постоянная активность.',
  'This has no Full members yet, so nothing here can go to a vote until one joins. Invite someone with Full standing.':
    'Здесь пока нет полноправных участников, поэтому ничего нельзя вынести на голосование, пока такой не присоединится. Пригласите кого-нибудь с полноправным статусом.',
  '1 Full member here decides its votes.': 'Голосования здесь решает 1 полноправный участник.',
  '{n} Full members here decide its votes.':
    'Голосования здесь решают полноправные участники: {n}.',
  'Only Full members vote. You get Full standing from two vouches by Full members, or after steady activity.':
    'Голосуют только полноправные участники. Полноправный статус дают два поручительства полноправных участников или постоянная активность.',
};

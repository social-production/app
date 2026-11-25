/** Russian, part of the catalogue (ADR 0114; see ../ru.ts): the profile ⋮ menu. */
import type { Entry } from '../languages.ts';

export const part39: Record<string, Entry> = {
  'More for {name}': 'Ещё: {name}',
  'Share profile': 'Поделиться профилем',
  Profile: 'Профиль',
  'Mark {name} as a bot': 'Отметить {name} как бота',
  'Already a full member.': 'Уже полноправный участник.',
  'A community review found this account is a bot.': 'Проверка сообщества показала, что это бот.',
  'Only full members can vouch.': 'Ручаться могут только полноправные участники.',
  'Opens once a full member vouches for you.':
    'Станет доступно, когда за вас поручится полноправный участник.',
  'Can vouch for people and mark bots.': 'Может ручаться за людей и отмечать ботов.',
  '{n} of 2 vouches; then they can vouch and mark bots.':
    '{n} из 2 поручительств; после этого можно ручаться и отмечать ботов.',
  'New members can start 10 a week ({n} left). Full members have no limit.':
    'Новые участники могут начать 10 в неделю (осталось {n}). У полноправных участников нет ограничений.',
};

import type { Moderation, ReportReason } from '#lib/domain/moderation.ts';
import {
  callCell,
  conversationCell,
  hashToString,
  stringToHash,
  type ActionHash,
} from '#lib/holo/index.ts';

import { toModeration, type ModerationWire } from './moderation.ts';

/**
 * Reports on direct and group chat messages (ADR 0075, web's `message` target): the
 * conversation DNA's `report_message`, `vote_report` and `message_moderation`, with the
 * same shapes as a space's (`moderation.ts`).
 */
const chat = async <T>(seed: string, fn: string, payload?: unknown): Promise<T> =>
  callCell<T>(await conversationCell(seed), 'messages', fn, payload);

/** The reported ones among a chat's messages, by id (the rest are visible). */
export async function chatModeration(
  seed: string,
  messages: string[],
): Promise<Record<string, Moderation>> {
  if (messages.length === 0) return {};
  const wire = await chat<{ id: ActionHash; moderation: ModerationWire }[]>(
    seed,
    'message_moderation',
    messages.map(stringToHash),
  );
  return Object.fromEntries(wire.map((m) => [hashToString(m.id), toModeration(m.moderation)]));
}

/** web's `submitReport` on a message, with your yes vote (in a 1:1 chat it removes it). */
export async function reportChatMessage(
  seed: string,
  message: string,
  reason: ReportReason,
  description: string,
): Promise<Moderation> {
  const wire = await chat<ModerationWire>(seed, 'report_message', {
    message: stringToHash(message),
    reason,
    description: description.trim(),
  });
  return toModeration(wire);
}

/** web's `setReportVote` on a message's report. */
export async function voteOnChatReport(
  seed: string,
  report: string,
  yes: boolean,
): Promise<Moderation> {
  const wire = await chat<ModerationWire>(seed, 'vote_report', {
    report: stringToHash(report),
    yes,
  });
  return toModeration(wire);
}

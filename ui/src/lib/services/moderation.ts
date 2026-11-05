import type { ContentReport, Moderation, ReportReason } from '#lib/domain/moderation.ts';
import {
  callCell,
  hashToString,
  stringToHash,
  type ActionHash,
  type AgentPubKey,
} from '#lib/holo/index.ts';

import { queued } from './chat.ts';
import { cellFor, type ProjectRef } from './projects.ts';

/**
 * Reports on a space (ADR 0071): the discussion zome's `report_space`, `vote_report`,
 * `moderation`; on its chat messages and comments (part b): `report_message`,
 * `message_moderation`.
 */
interface ReportWire {
  id: ActionHash;
  reporter: AgentPubKey;
  reason: string;
  description: string;
  created_at: number;
  resolution: ContentReport['resolution'];
  yes: number;
  no: number;
  mine: boolean | null;
  audience: number;
  delete_quorum: number;
  hide_quorum: number;
  delete_share: number;
  hide_share: number;
}

export interface ModerationWire {
  state: Moderation['state'];
  report: ReportWire | null;
}

function toReport(r: ReportWire): ContentReport {
  return {
    id: hashToString(r.id),
    reporter: hashToString(r.reporter),
    reason: r.reason === 'serious-harm' ? 'serious-harm' : 'spam',
    description: r.description,
    createdAt: r.created_at,
    resolution: r.resolution,
    yesCount: r.yes,
    noCount: r.no,
    activeVote: r.mine === null ? null : r.mine ? 'yes' : 'no',
    audienceSize: r.audience,
    deleteQuorum: r.delete_quorum,
    hideQuorum: r.hide_quorum,
    deleteYesShare: r.delete_share / 100,
    hideYesShare: r.hide_share / 100,
  };
}

export function toModeration(m: ModerationWire): Moderation {
  return { state: m.state, report: m.report ? toReport(m.report) : null };
}

/** The item's moderation state and its open (or removing) report; guests read it too. */
export async function getModeration(id: ProjectRef): Promise<Moderation> {
  return toModeration(
    await callCell<ModerationWire>(await cellFor(id), 'discussion', 'moderation'),
  );
}

/** web's `submitReport`: your report, with your yes vote. */
export async function reportItem(
  id: string,
  reason: ReportReason,
  description: string,
): Promise<Moderation> {
  const cell = await cellFor(id);
  const wire = await queued(() =>
    callCell<ModerationWire>(cell, 'discussion', 'report_space', {
      reason,
      description: description.trim(),
    }),
  );
  return toModeration(wire);
}

/** web's `setReportVote`: yes takes it down, no keeps it; a later vote replaces yours. */
export async function voteOnReport(id: string, report: string, yes: boolean): Promise<Moderation> {
  const cell = await cellFor(id);
  const wire = await queued(() =>
    callCell<ModerationWire>(cell, 'discussion', 'vote_report', {
      report: stringToHash(report),
      yes,
    }),
  );
  return toModeration(wire);
}

interface MessageModerationWire {
  id: ActionHash;
  moderation: ModerationWire;
}

/** The reported ones among a space's messages, by id (the rest are visible). */
export async function getMessageModeration(
  id: string,
  messages: string[],
): Promise<Record<string, Moderation>> {
  if (messages.length === 0) return {};
  const wire = await callCell<MessageModerationWire[]>(
    await cellFor(id),
    'discussion',
    'message_moderation',
    messages.map(stringToHash),
  );
  return Object.fromEntries(wire.map((m) => [hashToString(m.id), toModeration(m.moderation)]));
}

/** web's `submitReport` on a comment or chat message, with your yes vote. */
export async function reportMessage(
  id: string,
  message: string,
  reason: ReportReason,
  description: string,
): Promise<Moderation> {
  const cell = await cellFor(id);
  const wire = await queued(() =>
    callCell<ModerationWire>(cell, 'discussion', 'report_message', {
      message: stringToHash(message),
      reason,
      description: description.trim(),
    }),
  );
  return toModeration(wire);
}

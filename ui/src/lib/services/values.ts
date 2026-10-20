import type { Importance, ProjectValue } from '#lib/domain/values.ts';
import {
  callCell,
  hashToString,
  stringToHash,
  type ActionHash,
  type AgentPubKey,
} from '#lib/holo/index.ts';

import { cellOf } from './projects.ts';

interface ValueWire {
  id: ActionHash;
  author: AgentPubKey;
  created_at: number;
  label: string;
  count: number;
  average_x100: number;
  mine: Importance | null;
}

const governance = async <T>(projectId: string, fn: string, payload?: unknown): Promise<T> => {
  const { cell } = await cellOf(projectId);
  return callCell<T>(cell, 'governance', fn, payload);
};

/** Every value with its importance, oldest first. */
export async function listValues(projectId: string): Promise<ProjectValue[]> {
  const wire = await governance<ValueWire[]>(projectId, 'list_values');
  return wire.map((v) => ({
    id: hashToString(v.id),
    author: hashToString(v.author),
    createdAt: v.created_at,
    label: v.label,
    voteCount: v.count,
    importanceScore: v.average_x100 / 100,
    activeImportanceVote: v.mine ?? 0,
  }));
}

/** web's "Add value" (Proposal phase only). */
export async function addValue(projectId: string, label: string): Promise<void> {
  await governance<ActionHash>(projectId, 'add_value', label);
}

/** Rate a value's importance; the latest vote counts. */
export async function rateValue(
  projectId: string,
  value: string,
  importance: Importance,
): Promise<void> {
  await governance<ActionHash>(projectId, 'rate_value', {
    value: stringToHash(value),
    importance,
  });
}

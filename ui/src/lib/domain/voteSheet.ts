/**
 * The one vote sheet (owner, 2026-11-18: "we need some consistency"): a green kind line, a
 * plain headline saying what's being decided, the proposal, one count, Yes and No in the dock.
 */
import { t } from '#lib/i18n/index.ts';
import type { ProposalKind } from './governance.ts';

/** The kind line's first word: what sort of vote this is. */
export function voteKindLabel(kind: ProposalKind): string {
  if ('Edit' in kind) return t('Edit');
  if ('Update' in kind) return t('Update');
  if ('RemoveMember' in kind) return t('Member');
  if ('RequestSettings' in kind) return t('Requests');
  if ('Software' in kind) return kind.Software.label;
  if ('Convert' in kind) return t('Convert');
  return t('Phase change');
}

function editHeadline(e: Extract<ProposalKind, { Edit: unknown }>['Edit']): string {
  const changed = [e.title, e.description, e.location].filter((v) => v !== null).length;
  if (changed > 1) return t('New details');
  if (e.title !== null) return t('New title');
  if (e.description !== null) return t('New description');
  return t('New location');
}

/** The headline: what's being decided, in a few plain words. */
export function voteHeadline(kind: ProposalKind, phaseLabel: (id: string) => string): string {
  if ('Edit' in kind) return editHeadline(kind.Edit);
  if ('Update' in kind) return t('New update');
  if ('RemoveMember' in kind) return t('Remove member');
  if ('RequestSettings' in kind) return t('Request settings');
  if ('Software' in kind) return kind.Software.title || kind.Software.label;
  if ('Return' in kind) return t('Return to {phase}', { phase: phaseLabel(kind.Return.to) });
  if ('Convert' in kind) return t('Convert to a new proposal');
  return t('Move to {phase}', { phase: phaseLabel(kind.Phase.to) });
}

/** The phase a move leaves and the one it goes to, or null for other votes. */
export function phaseMove(
  kind: ProposalKind,
  from: string,
  phaseLabel: (id: string) => string,
): { from: string; to: string } | null {
  if ('Phase' in kind) return { from, to: phaseLabel(kind.Phase.to) };
  if ('Return' in kind) return { from, to: phaseLabel(kind.Return.to) };
  return null;
}

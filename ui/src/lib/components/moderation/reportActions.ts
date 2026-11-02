import { getContext, setContext } from 'svelte';

import type { Moderation, ReportReason } from '#lib/domain/moderation.ts';

/**
 * What the ⋮ report menu does (ADR 0071). Components stay presentational (CODE_RULES), so the
 * shell feature provides these once (`provideReportActions`) and every card's menu reads them.
 */
export interface ReportActions {
  /** The signed-in agent, or '' for guests. */
  me: () => string;
  /** web's requireViewer: true when signed in, else shows the sign-in notice. */
  requireViewer: (message: string) => boolean;
  load: (id: string) => Promise<Moderation>;
  report: (id: string, reason: ReportReason, description: string) => Promise<Moderation>;
  vote: (id: string, report: string, yes: boolean) => Promise<Moderation>;
  explain: (error: unknown) => string;
}

const KEY = Symbol('report-actions');

export function provideReportActions(actions: ReportActions): void {
  setContext(KEY, actions);
}

export function reportActions(): ReportActions {
  const actions = getContext<ReportActions | undefined>(KEY);
  if (!actions) throw new Error('ReportControl needs provideReportActions in the shell.');
  return actions;
}

import type { ReportReason } from '#lib/domain/moderation.ts';

/** What a space's chat does for web's bubble-menu Report and votes (ADR 0071 part b). */
export interface ChatReports {
  report: (message: string, reason: ReportReason, description: string) => Promise<void>;
  vote: (message: string, report: string, yes: boolean) => Promise<void>;
  explain: (error: unknown) => string;
  /** web's requireViewer: true when signed in, else shows the sign-in notice. */
  signedIn: () => boolean;
}

/** web's bubble menu shows the votes while the message's report is still open. */
export function openReport(
  moderation: { report: { id: string; resolution: string } | null } | undefined,
): string | null {
  const report = moderation?.report;
  if (!report || report.resolution === 'removed' || report.resolution === 'dismissed') return null;
  return report.id;
}

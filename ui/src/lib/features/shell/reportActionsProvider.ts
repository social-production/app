import { provideReportActions } from '#lib/components/moderation/reportActions.ts';
import { explain } from '#lib/services/errors.ts';
import { getModeration, reportItem, voteOnReport } from '#lib/services/moderation.ts';
import { requireViewer } from '#lib/stores/authNotice.svelte.ts';
import { connection } from '#lib/stores/connection.svelte.ts';

/** Give every ⋮ report menu below the shell its service calls (ADR 0071). */
export function provideReports(): void {
  provideReportActions({
    me: () => (connection.current.status === 'connected' ? connection.current.agent : ''),
    requireViewer,
    load: getModeration,
    report: reportItem,
    vote: voteOnReport,
    explain,
  });
}

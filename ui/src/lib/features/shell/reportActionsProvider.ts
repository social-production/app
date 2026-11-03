import { provideReportActions } from '#lib/components/moderation/reportActions.ts';
import { explain } from '#lib/services/errors.ts';
import {
  getMessageModeration,
  getModeration,
  reportItem,
  reportMessage,
  voteOnReport,
} from '#lib/services/moderation.ts';
import { VISIBLE } from '#lib/domain/moderation.ts';
import { requireViewer } from '#lib/stores/authNotice.svelte.ts';
import { connection } from '#lib/stores/connection.svelte.ts';

/** Give every ⋮ report menu below the shell its service calls (ADR 0071). */
export function provideReports(): void {
  provideReportActions({
    me: () => (connection.current.status === 'connected' ? connection.current.agent : ''),
    requireViewer,
    load: async (id, message) =>
      message
        ? ((await getMessageModeration(id, [message]))[message] ?? VISIBLE)
        : getModeration(id),
    report: (id, reason, description, message) =>
      message
        ? reportMessage(id, message, reason, description)
        : reportItem(id, reason, description),
    vote: voteOnReport,
    explain,
  });
}

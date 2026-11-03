import type { Moderation, ReportReason } from '#lib/domain/moderation.ts';
import { getMessageModeration, reportMessage, voteOnReport } from '#lib/services/moderation.ts';

/**
 * Reports on a space's chat messages (ADR 0071 part b, web's comment reports): which of the
 * chat's messages are reported, read when new messages arrive and every 30 s, and the report
 * and vote calls behind the bubble menu.
 */
export class ChatModeration {
  byId = $state<Record<string, Moderation>>({});
  private ids: string[] | null = null;

  constructor(private readonly space: string) {}

  /** Read the reports when the messages changed (or `force`, the 30 s re-read). */
  async refresh(ids: string[], force = false): Promise<void> {
    const known = this.ids;
    const same = !!known && ids.length === known.length && ids.every((id, i) => id === known[i]);
    if (same && !force) return;
    this.ids = ids;
    try {
      this.byId = await getMessageModeration(this.space, ids);
    } catch {
      // Unreadable reports leave the bubbles as they are until the next read.
    }
  }

  report = async (message: string, reason: ReportReason, description: string): Promise<void> => {
    const next = await reportMessage(this.space, message, reason, description);
    this.byId = { ...this.byId, [message]: next };
  };

  vote = async (message: string, report: string, yes: boolean): Promise<void> => {
    const next = await voteOnReport(this.space, report, yes);
    this.byId = { ...this.byId, [message]: next };
  };
}

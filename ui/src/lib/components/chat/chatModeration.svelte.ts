import type { Moderation, ReportReason } from '#lib/domain/moderation.ts';

/** The report calls of one chat: a space's (the project DNA's discussion zome) or a direct or
 * group chat's (the conversation DNA, ADR 0075). */
export interface ChatReportCalls {
  read: (messages: string[]) => Promise<Record<string, Moderation>>;
  report: (message: string, reason: ReportReason, description: string) => Promise<Moderation>;
  vote: (report: string, yes: boolean) => Promise<Moderation>;
}

/**
 * Reports on a chat's messages (ADR 0071 part b, ADR 0075; web's comment and message
 * reports): which of the chat's messages are reported, read when new messages arrive and
 * every 30 s, and the report and vote calls behind the bubble menu.
 */
export class ChatModeration {
  byId = $state<Record<string, Moderation>>({});
  private ids: string[] | null = null;

  constructor(private readonly calls: ChatReportCalls) {}

  /** Read the reports when the messages changed (or `force`, the 30 s re-read). */
  async refresh(ids: string[], force = false): Promise<void> {
    const known = this.ids;
    const same = !!known && ids.length === known.length && ids.every((id, i) => id === known[i]);
    if (same && !force) return;
    this.ids = ids;
    try {
      this.byId = ids.length === 0 ? {} : await this.calls.read(ids);
    } catch {
      // Unreadable reports leave the bubbles as they are until the next read.
    }
  }

  report = async (message: string, reason: ReportReason, description: string): Promise<void> => {
    const next = await this.calls.report(message, reason, description);
    this.byId = { ...this.byId, [message]: next };
  };

  vote = async (message: string, report: string, yes: boolean): Promise<void> => {
    const next = await this.calls.vote(report, yes);
    this.byId = { ...this.byId, [message]: next };
  };
}

import { startVisibilityPoll } from '#lib/components/actions/visibilityPoll.ts';
import type { PublicChat } from '#lib/domain/messages.ts';
import { explain } from '#lib/services/errors.ts';
import { myProjectChats } from '#lib/services/publicChats.ts';

/** web's INBOX_REFRESH_MS. */
const INBOX_REFRESH_MS = 8_000;

/** web's Public chat list: your projects' chats, newest first, refreshed while visible. */
export class PublicState {
  chats = $state<PublicChat[]>([]);
  loaded = $state(false);
  error = $state('');
  private stop: (() => void) | undefined;

  start(): void {
    if (this.stop) return;
    this.stop = startVisibilityPoll(() => this.refresh(), { activeMs: INBOX_REFRESH_MS });
    void this.refresh();
  }

  dispose(): void {
    this.stop?.();
    this.stop = undefined;
  }

  async refresh(): Promise<void> {
    try {
      const rows = await myProjectChats();
      this.chats = rows.sort((a, b) => b.activeAt - a.activeAt);
      this.error = '';
    } catch (e) {
      this.error = explain(e);
    } finally {
      this.loaded = true;
    }
  }

  /** What the list shows until the next refresh, once a chat has been read. */
  cleared(id: string): void {
    this.chats = this.chats.map((c) => (c.id === id ? { ...c, unread: 0 } : c));
  }

  unreadTotal = $derived(this.chats.reduce((sum, c) => sum + c.unread, 0));
}

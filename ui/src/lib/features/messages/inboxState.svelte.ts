import { startVisibilityPoll } from '#lib/components/actions/visibilityPoll.ts';
import type { ChatSummary, Contact, NewChat } from '#lib/domain/messages.ts';
import { chatSummary, contacts, myChats, sendNewChat } from '#lib/services/messages.ts';
import { explain } from '#lib/services/errors.ts';
import { myAgent, namesOf } from '#lib/services/people.ts';

import { inTurn } from './writes.ts';

/** web's INBOX_REFRESH_MS. */
const INBOX_REFRESH_MS = 8_000;

const lastActivity = (c: ChatSummary): number => c.last?.createdAt ?? c.invitedAt;

/**
 * web's Personal chat list: every chat in your inbox with its last message and unread count,
 * newest first, refreshed every 8 s while the page is visible.
 */
export class InboxState {
  chats = $state<ChatSummary[]>([]);
  names = $state<Record<string, string>>({});
  me = $state('');
  loaded = $state(false);
  error = $state('');
  private people: Promise<Contact[]> | undefined;
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
      this.me ||= await myAgent();
      const invites = await myChats();
      const settled = await Promise.allSettled(invites.map((i) => chatSummary(i.seed, i.at)));
      // A chat that didn't answer this time keeps its last row.
      const rows = settled.flatMap((r, i) => {
        if (r.status === 'fulfilled') return [r.value];
        const old = this.chats.find((c) => c.id === invites[i]?.seed);
        return old ? [old] : [];
      });
      this.chats = rows.sort((a, b) => lastActivity(b) - lastActivity(a));
      this.loaded = true;
      this.error = '';
      await this.loadNames();
    } catch (e) {
      this.error = explain(e);
      this.loaded = true;
    }
  }

  private async loadNames(): Promise<void> {
    const agents = this.chats.flatMap((c) => c.members);
    const missing = agents.filter((a, i) => agents.indexOf(a) === i && !(a in this.names));
    if (missing.length > 0) this.names = { ...this.names, ...(await namesOf(missing)) };
  }

  /** web's conversationDisplayTitle. */
  title(chat: ChatSummary): string {
    if (chat.title?.trim()) return chat.title.trim();
    if (chat.kind === 'direct') {
      const partner = chat.members.find((m) => m !== this.me) ?? chat.members[0];
      return partner ? (this.names[partner] ?? '…') : 'Direct message';
    }
    return 'Group chat';
  }

  unreadTotal = $derived(this.chats.reduce((sum, c) => sum + c.unread, 0));

  /** People matching `query` by name, not yourself (web's getMessageContacts, 8 at most). */
  async search(query: string): Promise<Contact[]> {
    this.people ??= contacts();
    const needle = query.trim().toLowerCase();
    let everyone: Contact[] = [];
    try {
      everyone = await this.people;
    } catch {
      this.people = undefined; // try the directory again on the next keystroke
    }
    return everyone
      .filter((c) => c.agent !== this.me && c.name.toLowerCase().includes(needle))
      .slice(0, 8);
  }

  /** Start a chat (or reuse your direct chat with that person); returns its id. */
  async send(chat: NewChat): Promise<string> {
    try {
      const id = await inTurn(() => sendNewChat(chat, this.chats));
      await this.refresh();
      return id;
    } catch (e) {
      throw new Error(explain(e), { cause: e });
    }
  }
}

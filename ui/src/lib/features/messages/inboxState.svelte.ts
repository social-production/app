import { startVisibilityPoll } from '#lib/components/actions/visibilityPoll.ts';
import type { ChatSummary, Contact, ListPrefs, NewChat } from '#lib/domain/messages.ts';
import {
  addGroupMember,
  chatSummary,
  contacts,
  leaveGroup,
  myChats,
  removeGroupMember,
  renameGroup,
  sendNewChat,
  setChatPrefs,
} from '#lib/services/messages.ts';
import { explain } from '#lib/services/errors.ts';
import { myAgent, namesOf } from '#lib/services/people.ts';

import { inTurn } from './writes.ts';

/** web's INBOX_REFRESH_MS. */
const INBOX_REFRESH_MS = 8_000;

const lastActivity = (c: ChatSummary): number => c.last?.createdAt ?? c.invitedAt;

/**
 * web's Personal chat list: every chat in your inbox with its last message and unread count,
 * pinned first, then newest, refreshed every 8 s while the page is visible. Chats you left,
 * were removed from or deleted (until something new arrives) are left out, as on web.
 */
export class InboxState {
  /** Every chat in your inbox, shown or not (an open chat stays open while you leave it). */
  all = $state<ChatSummary[]>([]);
  chats = $derived(this.all.filter((c) => c.member && !c.hidden));
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
        const old = this.all.find((c) => c.id === invites[i]?.seed);
        return old ? [old] : [];
      });
      // web's byPinnedThenRecent.
      this.all = rows.sort((a, b) => +b.pinned - +a.pinned || lastActivity(b) - lastActivity(a));
      this.loaded = true;
      this.error = '';
      await this.loadNames();
    } catch (e) {
      this.error = explain(e);
      this.loaded = true;
    }
  }

  private async loadNames(): Promise<void> {
    const agents = this.all.flatMap((c) => [...c.members, ...(c.last ? [c.last.author] : [])]);
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

  /** The list's preview line; a leave reads "<name> left the group." */
  preview(chat: ChatSummary): string {
    if (chat.last?.kind === 'system')
      return `${this.names[chat.last.author] ?? '…'} left the group.`;
    return chat.last?.preview ?? '';
  }

  /** Run a chat write in turn, then refresh; errors come back as plain text. */
  private async write(fn: () => Promise<void>): Promise<void> {
    try {
      await inTurn(fn);
    } catch (e) {
      throw new Error(explain(e), { cause: e });
    } finally {
      await this.refresh();
    }
  }

  /** web's chat menu: pin, mute, delete (hide) for you alone. */
  setPrefs(id: string, prefs: ListPrefs): Promise<void> {
    return this.write(() => setChatPrefs(id, prefs));
  }

  leave(id: string): Promise<void> {
    return this.write(() => leaveGroup(id));
  }

  rename(id: string, title: string): Promise<void> {
    return this.write(() => renameGroup(id, title));
  }

  addMember(id: string, agent: string): Promise<void> {
    return this.write(() => addGroupMember(id, agent));
  }

  removeMember(id: string, agent: string): Promise<void> {
    return this.write(() => removeGroupMember(id, agent));
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
      const id = await inTurn(() => sendNewChat(chat, this.all));
      await this.refresh();
      return id;
    } catch (e) {
      throw new Error(explain(e), { cause: e });
    }
  }
}

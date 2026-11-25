import { setGroupManager, setGroupPicture } from '#lib/services/groupAdmin.ts';
import { t } from '#lib/i18n/index.ts';
import { HOLDER_PREFIX } from '#lib/domain/serviceRequests.ts';
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
      // A collective request's holder chat is reached from its service, not Messages (ADR 0113).
      const shown = rows.filter((c) => !(c.kind === 'group' && c.title?.startsWith(HOLDER_PREFIX)));
      // web's byPinnedThenRecent.
      this.all = shown.sort((a, b) => +b.pinned - +a.pinned || lastActivity(b) - lastActivity(a));
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
      return partner ? (this.names[partner] ?? '…') : t('Direct message');
    }
    return t('Group chat');
  }

  /** The other person in a direct chat (their face on the row); none for groups. */
  partner(chat: ChatSummary): string | null {
    if (chat.kind !== 'direct') return null;
    return chat.members.find((m) => m !== this.me) ?? null;
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

  setManager(id: string, agent: string, on: boolean): Promise<void> {
    return this.write(() => setGroupManager(id, agent, on));
  }

  setPicture(id: string, photo: Blob): Promise<void> {
    return this.write(() => setGroupPicture(id, photo));
  }

  unreadTotal = $derived(this.chats.reduce((sum, c) => sum + c.unread, 0));

  /** Everyone with a profile but yourself, for New message's list and group members. */
  async everyone(): Promise<Contact[]> {
    this.people ??= contacts();
    try {
      this.me ||= await myAgent();
      return (await this.people).filter((c) => c.agent !== this.me);
    } catch (e) {
      this.people = undefined; // read the directory again next time
      throw e;
    }
  }

  /** People matching `query` by name, not yourself (web's getMessageContacts, 8 at most). */
  async search(query: string): Promise<Contact[]> {
    const needle = query.trim().toLowerCase();
    const everyone = await this.everyone().catch(() => []);
    return everyone.filter((c) => c.name.toLowerCase().includes(needle)).slice(0, 8);
  }

  /** Your direct chat with this person, if you have one. */
  directWith(agent: string): ChatSummary | null {
    return (
      this.chats.find(
        (c) => c.kind === 'direct' && c.members.length === 2 && c.members.includes(agent),
      ) ?? null
    );
  }

  /** Start a chat (or reuse your direct chat with that person); returns its id. */
  async send(chat: NewChat, files: File[] = []): Promise<string> {
    try {
      const id = await inTurn(() => sendNewChat(chat, this.all, files));
      await this.refresh();
      return id;
    } catch (e) {
      throw new Error(explain(e), { cause: e });
    }
  }
}

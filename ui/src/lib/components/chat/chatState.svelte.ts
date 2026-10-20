import { startVisibilityPoll } from '#lib/components/actions/visibilityPoll.ts';
import type { ChatLine, ChatMessage } from '#lib/domain/chat.ts';

/** Where a chat's messages come from: a project chat, or a direct or group chat. Features
 * pass services in, since components don't call them. */
export interface ChatSource {
  list: () => Promise<ChatMessage[]>;
  post: (body: string, replyTo: string | null) => Promise<unknown>;
  remove: (id: string) => Promise<void>;
  /** Direct and group chats only (web's conversationActions): edit your own, pin. */
  edit?: (id: string, body: string) => Promise<void>;
  /** Set a message pinned or not. */
  pin?: (id: string, pinned: boolean) => Promise<void>;
  /** The viewer: their key, name, and whether their account is paused. */
  viewer: () => Promise<{ agent: string; name: string; paused: boolean }>;
  /** Display names for these agents. */
  names: (agents: string[]) => Promise<Record<string, string>>;
  explain: (e: unknown) => string;
  /** Called after each load that brought something (a direct chat marks itself read). */
  seen?: () => void;
}

interface Pending {
  id: string;
  body: string;
  replyTo: ChatLine | null;
  at: number;
}

/**
 * One chat's state (web's ProjectChatTab and MessagesPage threads): the newest page from the
 * zome, copies of messages being sent, and polling while it is open (ADR 0038: 5 s open,
 * 45 s otherwise).
 */
export class ChatState {
  messages = $state<ChatMessage[]>([]);
  pending = $state<Pending[]>([]);
  names = $state<Record<string, string>>({});
  me = $state('');
  paused = $state(false);
  /** Until the first load, an empty chat means "still loading", not "no messages". */
  loaded = $state(false);
  error = $state('');
  private stop: (() => void) | undefined;
  /** Who the viewer is: a copy being sent carries their key (and links to their profile). */
  private ready: Promise<void> = Promise.resolve();

  constructor(
    private readonly source: ChatSource,
    known: Record<string, string>,
  ) {
    this.names = { ...known };
  }

  lines: ChatLine[] = $derived([
    ...this.messages.map((m) => this.line(m)),
    ...this.pending.map((p) => this.pendingLine(p)),
  ]);

  private nameOf(agent: string | null): string {
    return agent ? (this.names[agent] ?? '…') : 'Message';
  }

  private line(m: ChatMessage): ChatLine {
    const quote = m.replyTo;
    return {
      id: m.id,
      author: m.author,
      authorName: this.nameOf(m.author),
      body: m.body,
      createdAt: m.createdAt / 1000,
      own: m.author === this.me,
      pending: false,
      replyToId: quote?.id ?? null,
      replyAuthor: quote ? this.nameOf(quote.author) : null,
      // web keeps a deleted original's place as "Deleted message".
      replyPreview: quote ? (quote.deleted ? 'Deleted message' : quote.preview) : null,
      kind: m.kind ?? 'text',
      edited: m.edited ?? false,
      pinned: m.pinned ?? false,
    };
  }

  private pendingLine(p: Pending): ChatLine {
    return {
      id: p.id,
      author: this.me,
      authorName: this.nameOf(this.me),
      body: p.body,
      createdAt: p.at,
      own: true,
      pending: true,
      replyToId: p.replyTo?.id ?? null,
      replyAuthor: p.replyTo?.authorName ?? null,
      replyPreview: p.replyTo ? p.replyTo.body.slice(0, 120) : null,
      kind: 'text',
      edited: false,
      pinned: false,
    };
  }

  /** Load now and keep polling; `isActive` says whether the chat tab is showing. */
  start(isActive: () => boolean): void {
    if (this.stop) return;
    this.stop = startVisibilityPoll(() => this.refresh(), {
      activeMs: 5_000,
      idleMs: 45_000,
      isActive,
    });
    this.ready = this.whoAmI();
    void this.refresh();
  }

  dispose(): void {
    this.stop?.();
    this.stop = undefined;
  }

  private async whoAmI(): Promise<void> {
    try {
      const viewer = await this.source.viewer();
      this.me = viewer.agent;
      this.names = { ...this.names, [this.me]: viewer.name };
      this.paused = viewer.paused;
    } catch {
      // Names and the paused notice fill in on the next refresh.
    }
  }

  async refresh(): Promise<void> {
    try {
      const newest = this.messages.at(-1)?.id;
      this.messages = await this.source.list();
      this.loaded = true;
      if (this.messages.at(-1)?.id !== newest) this.source.seen?.();
      await this.loadNames();
    } catch {
      // Keep what is shown until the next successful refresh.
    }
  }

  private async loadNames(): Promise<void> {
    const agents = this.messages.flatMap((m) => [m.author, m.replyTo?.author ?? null]);
    const missing = agents.filter(
      (a, i): a is string => !!a && agents.indexOf(a) === i && !(a in this.names),
    );
    if (missing.length === 0) return;
    this.names = { ...this.names, ...(await this.source.names(missing)) };
  }

  /** Show a copy at once, write it, then reload; a failure removes the copy and keeps the draft. */
  async send(body: string, replyToId: string | null): Promise<void> {
    if (!this.me) await this.ready;
    const replyTo = this.lines.find((l) => l.id === replyToId) ?? null;
    const copy: Pending = { id: `pending-${crypto.randomUUID()}`, body, replyTo, at: Date.now() };
    this.pending = [...this.pending, copy];
    this.error = '';
    try {
      await this.source.post(body, replyToId);
      await this.refresh();
    } catch (e) {
      this.error = this.source.explain(e);
      throw e;
    } finally {
      this.pending = this.pending.filter((p) => p.id !== copy.id);
    }
  }

  /** web's editConversationMessage: throws (keeping the draft) when it fails. */
  async edit(id: string, body: string): Promise<void> {
    this.error = '';
    try {
      await this.source.edit?.(id, body);
    } catch (e) {
      this.error = this.source.explain(e) || 'Could not edit message';
      throw e;
    }
    await this.refresh();
  }

  /** web's toggleConversationPin: `pinned` is the message's current state, so this flips it. */
  async pin(id: string, pinned: boolean): Promise<void> {
    this.error = '';
    try {
      await this.source.pin?.(id, !pinned);
    } catch (e) {
      this.error = this.source.explain(e) || 'Could not update the pin';
    }
    await this.refresh();
  }

  async remove(id: string): Promise<void> {
    this.error = '';
    try {
      await this.source.remove(id);
    } catch {
      this.error = 'Could not delete that message';
    }
    await this.refresh();
  }
}

import { startVisibilityPoll } from '#lib/components/actions/visibilityPoll.ts';
import type { ChatLine, ChatMessage } from '#lib/domain/chat.ts';
import { deleteMessage, listMessages, postMessage } from '#lib/services/chat.ts';
import { explain } from '#lib/services/errors.ts';
import { getPerson, myAgent } from '#lib/services/people.ts';

interface Pending {
  id: string;
  body: string;
  replyTo: ChatLine | null;
  at: number;
}

/**
 * One project's chat (web's ProjectChatTab state): the newest page from the zome, copies of
 * messages being sent, and polling while the tab is open (ADR 0038: 5 s open, 45 s otherwise).
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

  constructor(
    readonly projectId: string,
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
    void this.whoAmI();
    void this.refresh();
  }

  dispose(): void {
    this.stop?.();
  }

  private async whoAmI(): Promise<void> {
    try {
      this.me = await myAgent();
      const person = await getPerson(this.me);
      this.names = { ...this.names, [this.me]: person.profile?.name ?? 'Unnamed' };
      this.paused = person.standing.level === 'Paused';
    } catch {
      // Names and the paused notice fill in on the next refresh.
    }
  }

  async refresh(): Promise<void> {
    try {
      this.messages = await listMessages(this.projectId);
      this.loaded = true;
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
    const people = await Promise.all(missing.map(getPerson));
    const names = { ...this.names };
    for (const p of people) names[p.agent] = p.profile?.name ?? 'Unnamed';
    this.names = names;
  }

  /** Show a copy at once, write it, then reload; a failure removes the copy and keeps the draft. */
  async send(body: string, replyToId: string | null): Promise<void> {
    const replyTo = this.lines.find((l) => l.id === replyToId) ?? null;
    const copy: Pending = { id: `pending-${crypto.randomUUID()}`, body, replyTo, at: Date.now() };
    this.pending = [...this.pending, copy];
    this.error = '';
    try {
      await postMessage(this.projectId, body, replyToId);
      await this.refresh();
    } catch (e) {
      this.error = explain(e);
      throw e;
    } finally {
      this.pending = this.pending.filter((p) => p.id !== copy.id);
    }
  }

  async remove(id: string): Promise<void> {
    this.error = '';
    try {
      await deleteMessage(this.projectId, id);
    } catch {
      this.error = 'Could not delete that message';
    }
    await this.refresh();
  }
}

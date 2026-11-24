import type { CommentSort, DiscussionComment, VoteDirection } from '#lib/domain/discussions.ts';
import type { Moderation } from '#lib/domain/moderation.ts';
import type { Project, Stance } from '#lib/domain/projects.ts';
import { queued } from '#lib/services/chat.ts';
import { addComment, listComments, voteComment } from '#lib/services/discussions.ts';
import { explain } from '#lib/services/errors.ts';
import { getPerson, myAgent } from '#lib/services/people.ts';
import { getProject, setSignal } from '#lib/services/projects.ts';
import { noteComment } from '#lib/services/profileNotes.ts';
import { requireViewer } from '#lib/stores/authNotice.svelte.ts';
import { overHeld, withSignal, type HeldSignal } from '../heldSignal.ts';
import { ModerationLoader } from '../moderationState.svelte.ts';

/** Every comment in the tree with `id` changed by `change`. */
function mapTree(
  list: DiscussionComment[],
  id: string,
  change: (c: DiscussionComment) => DiscussionComment,
): DiscussionComment[] {
  return list.map((c) =>
    c.id === id ? change(c) : { ...c, replies: mapTree(c.replies, id, change) },
  );
}

/** The author of comment `id` anywhere in the tree. */
function authorOf(list: DiscussionComment[], id: string): string | null {
  for (const c of list) {
    if (c.id === id) return c.author;
    const found = authorOf(c.replies, id);
    if (found) return found;
  }
  return null;
}

function authors(list: DiscussionComment[]): string[] {
  return list.flatMap((c) => [c.author, ...authors(c.replies)]);
}

/** One thread or post page (web's ThreadPageData / PostPageData, ADR 0070). */
export class DiscussionState {
  project = $state<Project>();
  #held: HeldSignal | null = null;
  comments = $state<DiscussionComment[]>([]);
  names = $state<Record<string, string>>({});
  sortMode = $state<CommentSort>('oldest');
  error = $state<string>();
  message = $state('');

  /** web's ⋮ report menu and the removed notice (ADR 0071). */
  readonly moderation: ModerationLoader;

  constructor(readonly id: string) {
    this.moderation = new ModerationLoader(id);
  }

  /** First load: a thread or post just created may take a few seconds to reach this node. */
  async open(): Promise<void> {
    void this.moderation.load();
    for (let attempt = 1; ; attempt++) {
      try {
        await this.fetch();
        return;
      } catch (e) {
        if (attempt >= 15) {
          this.error = explain(e);
          return;
        }
      }
      await new Promise((resolve) => setTimeout(resolve, 2000));
    }
  }

  async fetch(): Promise<void> {
    const [project, comments] = await Promise.all([getProject(this.id), listComments(this.id)]);
    [this.project, this.#held] = overHeld(project, this.#held);
    this.comments = comments;
    this.error = undefined;
    const agents = [project.creator, ...authors(comments)];
    const missing = agents.filter((a, i) => agents.indexOf(a) === i && !(a in this.names));
    const people = await Promise.all(missing.map(getPerson));
    this.names = {
      ...this.names,
      ...Object.fromEntries(people.map((p) => [p.agent, p.profile?.name ?? 'Unnamed'])),
    };
  }

  /** web's vote on the thread or post: its Support/Oppose signal. */
  async vote(next: Stance | null): Promise<void> {
    if (!requireViewer('Sign in to vote.')) throw new Error('signed out');
    // One write at a time per chain: comments and comment votes share this queue.
    await queued(() => setSignal(this.id, next));
    if (this.project) [this.project, this.#held] = withSignal(this.project, next);
    void this.fetch().catch(() => undefined);
  }

  /** web's comment vote (`direction` is the new vote); the strip rolls back if this throws. */
  async voteComment(comment: string, direction: VoteDirection): Promise<void> {
    if (!requireViewer('Sign in to vote.')) throw new Error('signed out');
    await voteComment(this.id, comment, direction);
    this.comments = mapTree(this.comments, comment, (c) => ({
      ...c,
      activeVote: direction,
      voteCount: c.voteCount - c.activeVote + direction,
    }));
  }

  /** A report or vote on a comment (ADR 0071 part b) shows at once. */
  setCommentModeration(comment: string, next: Moderation): void {
    this.comments = mapTree(this.comments, comment, (c) => ({ ...c, moderation: next }));
  }

  /** A comment shown at once, marked "Sending…", until the page reads it back. */
  private async pending(parent: string | null, body: string): Promise<void> {
    const author = await myAgent().catch(() => '');
    const draft: DiscussionComment = {
      id: `sending-${Date.now()}`,
      author,
      body: body.trim(),
      createdAt: Date.now() * 1000,
      voteCount: 0,
      activeVote: 0,
      replies: [],
      sending: true,
    };
    this.comments = parent
      ? mapTree(this.comments, parent, (c) => ({ ...c, replies: [...c.replies, draft] }))
      : [...this.comments, draft];
  }

  /** web's addComment: true once written (the composer then clears). */
  async comment(parent: string | null, body: string): Promise<boolean> {
    if (!requireViewer('Sign in to comment.') || !body.trim()) return false;
    this.message = '';
    const shown = this.pending(parent, body);
    try {
      const id = await addComment(this.id, body, parent);
      // ADR 0107: the note lists it on your profile and followers' Home.
      await noteComment(this.id, id, body, parent ? authorOf(this.comments, parent) : null);
      await this.fetch();
      return true;
    } catch (e) {
      await shown;
      this.comments = dropSending(this.comments);
      this.message = explain(e);
      return false;
    }
  }
}

function dropSending(list: DiscussionComment[]): DiscussionComment[] {
  return list.filter((c) => !c.sending).map((c) => ({ ...c, replies: dropSending(c.replies) }));
}

import type { CommentSort, DiscussionComment, VoteDirection } from '#lib/domain/discussions.ts';
import type { Project, Stance } from '#lib/domain/projects.ts';
import { addComment, listComments, voteComment } from '#lib/services/discussions.ts';
import { explain } from '#lib/services/errors.ts';
import { getPerson } from '#lib/services/people.ts';
import { getProject, setSignal } from '#lib/services/projects.ts';
import { requireViewer } from '#lib/stores/authNotice.svelte.ts';

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

function authors(list: DiscussionComment[]): string[] {
  return list.flatMap((c) => [c.author, ...authors(c.replies)]);
}

/** One thread or post page (web's ThreadPageData / PostPageData, ADR 0070). */
export class DiscussionState {
  project = $state<Project>();
  comments = $state<DiscussionComment[]>([]);
  names = $state<Record<string, string>>({});
  sortMode = $state<CommentSort>('oldest');
  error = $state<string>();
  message = $state('');

  constructor(readonly id: string) {}

  /** First load: a thread or post just created may take a few seconds to reach this node. */
  async open(): Promise<void> {
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
    this.project = project;
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
    await setSignal(this.id, next);
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

  /** web's addComment: true once written (the composer then clears). */
  async comment(parent: string | null, body: string): Promise<boolean> {
    if (!requireViewer('Sign in to comment.') || !body.trim()) return false;
    this.message = '';
    try {
      await addComment(this.id, body, parent);
      await this.fetch();
      return true;
    } catch (e) {
      this.message = explain(e);
      return false;
    }
  }
}

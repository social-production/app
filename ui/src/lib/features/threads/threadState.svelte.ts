import {
  commentTree,
  sortComments,
  type CommentNode,
  type CommentSort,
  type ThreadDetail,
  type VoteDirection,
} from '#lib/domain/threads.ts';
import { explain } from '#lib/services/errors.ts';
import { getProject } from '#lib/services/projects.ts';
import { addComment, castVote, getThread } from '#lib/services/threads.ts';

import { viewer, withNames } from './people.ts';

/** One thread page (web's ThreadDetailPage data): the thread, its comments, names, sort. */
export class ThreadState {
  detail = $state<ThreadDetail | null>(null);
  projectTitle = $state('');
  names = $state<Record<string, string>>({});
  sort = $state<CommentSort>('oldest');
  paused = $state(false);
  /** A load failure before anything showed (the thread page's message). */
  error = $state('');
  /** A comment that couldn't be written (shown above the comments; the draft stays). */
  writeError = $state('');

  constructor(
    readonly projectId: string,
    readonly threadId: string,
  ) {}

  comments: CommentNode[] = $derived(
    sortComments(
      commentTree(this.detail?.comments ?? [], (a) => this.names[a] ?? '…'),
      this.sort,
    ),
  );

  async open(): Promise<void> {
    void viewer()
      .then((v) => {
        this.paused = v.paused;
        this.names = { ...this.names, [v.me]: v.name };
      })
      .catch(() => undefined);
    void getProject(this.projectId)
      .then((p) => (this.projectTitle = p.charter?.title ?? ''))
      .catch(() => undefined);
    await this.reload();
  }

  async reload(): Promise<void> {
    try {
      const detail = await getThread(this.projectId, this.threadId);
      this.detail = detail;
      this.error = '';
      this.names = await withNames(
        [detail.thread.author, ...detail.comments.map((c) => c.author)],
        this.names,
      );
    } catch (e) {
      if (!this.detail) this.error = explain(e);
    }
  }

  /** The strip has already moved; a failure throws so it rolls back. */
  async vote(target: string, next: VoteDirection): Promise<void> {
    await castVote(this.projectId, target, next);
  }

  /** A comment (or a reply to `parent`); true once written. */
  async comment(body: string, parent: string | null = null): Promise<boolean> {
    this.writeError = '';
    try {
      await addComment(this.projectId, this.threadId, parent, body);
    } catch (e) {
      this.writeError = explain(e);
      return false;
    }
    await this.reload();
    return true;
  }
}

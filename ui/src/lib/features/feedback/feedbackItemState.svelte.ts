import type { FeedbackComment, FeedbackItem, FeedbackVote } from '#lib/domain/feedback.ts';
import { explain } from '#lib/services/errors.ts';
import {
  addFeedbackComment,
  feedbackComments,
  getFeedback,
  likeFeedbackComment,
  voteFeedback,
} from '#lib/services/feedback.ts';
import { namesOf } from '#lib/services/people.ts';
import { requireViewer } from '#lib/stores/authNotice.svelte.ts';

/** One feedback item's page: the item, its votes and its flat comments (Plan, Phase and
 * Feedback Screens §7). */
export class FeedbackItemState {
  item = $state<FeedbackItem | null>(null);
  comments = $state<FeedbackComment[]>([]);
  names = $state<Record<string, string>>({});
  loaded = $state(false);
  error = $state('');
  busy = $state(false);

  constructor(private readonly id: string) {}

  async load(): Promise<void> {
    try {
      const [item, comments] = await Promise.all([getFeedback(this.id), feedbackComments(this.id)]);
      this.item = item;
      this.comments = comments;
      await this.learnNames();
      this.error = item ? '' : 'This feedback was not found.';
    } catch (e) {
      this.error = explain(e) || 'Could not load this feedback.';
    } finally {
      this.loaded = true;
    }
  }

  async vote(vote: FeedbackVote): Promise<void> {
    if (!requireViewer('Sign in to vote on feedback.')) return;
    try {
      const next = await voteFeedback(this.id, vote);
      if (next) this.item = next;
    } catch (e) {
      this.error = explain(e);
    }
  }

  /** Post a comment (an answer to `replyTo`, if any); true when it went through. */
  async comment(body: string, replyTo: string | null): Promise<boolean> {
    if (!body.trim() || this.busy) return false;
    this.busy = true;
    try {
      const added = await addFeedbackComment(this.id, body.trim(), replyTo);
      this.comments = [...this.comments.filter((c) => c.id !== added.id), added];
      if (this.item) this.item = { ...this.item, comments: this.item.comments + 1 };
      await this.learnNames();
      return true;
    } catch (e) {
      this.error = explain(e) || 'Could not post the comment.';
      return false;
    } finally {
      this.busy = false;
    }
  }

  async like(comment: FeedbackComment): Promise<void> {
    if (!requireViewer('Sign in to like comments.')) return;
    // Shown at once; the reply carries the network's view.
    this.replace({
      ...comment,
      liked: !comment.liked,
      likes: comment.likes + (comment.liked ? -1 : 1),
    });
    try {
      const next = await likeFeedbackComment(comment.id, !comment.liked);
      if (next) this.replace(next);
    } catch (e) {
      this.replace(comment);
      this.error = explain(e);
    }
  }

  private replace(next: FeedbackComment): void {
    this.comments = this.comments.map((c) => (c.id === next.id ? next : c));
  }

  private async learnNames(): Promise<void> {
    const agents = [this.item?.author, ...this.comments.map((c) => c.author)].filter(
      (a): a is string => !!a && !(a in this.names),
    );
    if (agents.length > 0)
      this.names = {
        ...this.names,
        ...(await namesOf(agents.filter((a, i) => agents.indexOf(a) === i))),
      };
  }
}

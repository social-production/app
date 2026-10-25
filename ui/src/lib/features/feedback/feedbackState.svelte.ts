import type { FeedbackInput, FeedbackItem, FeedbackVote } from '#lib/domain/feedback.ts';
import { withVote } from '#lib/domain/feedback.ts';
import { explain } from '#lib/services/errors.ts';
import { createFeedback, getFeedback, listFeedback, voteFeedback } from '#lib/services/feedback.ts';
import { namesOf } from '#lib/services/people.ts';
import { requireViewer } from '#lib/stores/authNotice.svelte.ts';

/** web's FeedbackPage state: the list, the compose and detail sheets, and votes shown at once
 *  (rolled back if the write fails). */
export class FeedbackState {
  items = $state<FeedbackItem[]>([]);
  names = $state<Record<string, string>>({});
  loading = $state(true);
  error = $state('');
  composeOpen = $state(false);
  composePending = $state(false);
  composeMessage = $state('');
  detailId = $state<string | null>(null);
  voting = $state<string[]>([]);

  readonly detail = $derived(this.items.find((i) => i.id === this.detailId) ?? null);

  async load(): Promise<void> {
    try {
      this.items = await listFeedback();
      this.error = '';
      void this.learnNames();
    } catch {
      this.error = 'Could not load feedback right now.';
    } finally {
      this.loading = false;
    }
  }

  compose(): void {
    if (!requireViewer('Sign in to share feedback.')) return;
    this.composeMessage = '';
    this.composeOpen = true;
  }

  async submit(input: FeedbackInput): Promise<void> {
    this.composePending = true;
    this.composeMessage = '';
    try {
      const item = await createFeedback(input);
      this.items = [item, ...this.items.filter((i) => i.id !== item.id)];
      this.composeOpen = false;
      void this.learnNames();
    } catch (e) {
      this.composeMessage = explain(e) || 'Could not submit feedback.';
    } finally {
      this.composePending = false;
    }
  }

  async open(id: string): Promise<void> {
    this.detailId = id;
    const fresh = await getFeedback(id).catch(() => null);
    if (fresh) this.merge(fresh);
  }

  async vote(id: string, vote: FeedbackVote): Promise<void> {
    if (!requireViewer('Sign in to vote on platform feedback.')) return;
    const before = this.items.find((i) => i.id === id);
    if (!before) return;
    this.merge(withVote(before, vote));
    this.voting = [...this.voting, id];
    try {
      const after = await voteFeedback(id, vote);
      if (after) this.merge(after);
    } catch {
      this.merge(before);
    } finally {
      this.voting = this.voting.filter((v) => v !== id);
    }
  }

  private merge(item: FeedbackItem): void {
    this.items = this.items.map((i) => (i.id === item.id ? item : i));
  }

  private async learnNames(): Promise<void> {
    const missing = this.items.map((i) => i.author).filter((a, n, all) => all.indexOf(a) === n);
    const unknown = missing.filter((a) => !(a in this.names));
    if (unknown.length > 0) this.names = { ...this.names, ...(await namesOf(unknown)) };
  }
}

import type { BotReview } from '#lib/domain/botReview.ts';
import type { Person } from '#lib/domain/people.ts';
import {
  getBotReview,
  requestBotReReview,
  setBotMark,
  voteBotReview,
} from '#lib/services/botReview.ts';
import { explain } from '#lib/services/errors.ts';
import { getPerson } from '#lib/services/people.ts';

/** web's ProfilePage trust state (ADR 0074): the bot review card, the Bot action and the
 * Trust sheet's Vouchers / Bot marks. `onChange` reloads the profile after a write. */
export class ProfileTrust {
  review = $state<BotReview>();
  vouchers = $state<Person[]>([]);
  markers = $state<Person[]>([]);
  open = $state(false);
  confirming = $state(false);
  stancePending = $state(false);
  reviewPending = $state(false);
  stanceMessage = $state('');
  reviewMessage = $state('');

  constructor(
    private readonly agent: string,
    private readonly onChange: () => Promise<void>,
  ) {}

  async load(person: Person): Promise<void> {
    try {
      const review = await getBotReview(this.agent);
      const [vouchers, markers] = await Promise.all([
        Promise.all(person.standing.vouchers.map(getPerson)),
        Promise.all(review.markers.map(getPerson)),
      ]);
      this.review = review;
      this.vouchers = vouchers;
      this.markers = markers;
    } catch {
      // Profiles still show without the review (an older seed node has no such call).
    }
  }

  markedBy(viewer: string): boolean {
    return this.review?.markers.includes(viewer) ?? false;
  }

  /** web's trustAction('bot'): clear your mark, or confirm first, then mark. */
  toggleMark(viewer: string): void {
    if (this.stancePending) return;
    if (this.markedBy(viewer)) void this.setMark(false);
    else this.confirming = true;
  }

  async setMark(bot: boolean): Promise<void> {
    this.confirming = false;
    this.stancePending = true;
    this.stanceMessage = '';
    try {
      await setBotMark(this.agent, bot);
      await this.onChange();
    } catch (e) {
      this.stanceMessage = explain(e) || 'Could not update that stance.';
    } finally {
      this.stancePending = false;
    }
  }

  async vote(vote: 'bot' | 'real'): Promise<void> {
    if (!this.review || this.review.myVote === vote) return;
    this.reviewPending = true;
    this.reviewMessage = '';
    try {
      await voteBotReview(this.agent, vote);
      await this.onChange();
    } catch (e) {
      this.reviewMessage = explain(e) || 'Could not record that vote.';
    } finally {
      this.reviewPending = false;
    }
  }

  async reReview(): Promise<void> {
    this.reviewPending = true;
    this.reviewMessage = '';
    try {
      await requestBotReReview();
      await this.onChange();
    } catch (e) {
      this.reviewMessage = explain(e) || 'Could not ask for a re-review.';
    } finally {
      this.reviewPending = false;
    }
  }
}

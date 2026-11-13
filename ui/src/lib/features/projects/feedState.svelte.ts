import type { FeedSort, FeedWindow, ProjectCard, Stance } from '#lib/domain/projects.ts';
import { explain } from '#lib/services/errors.ts';
import { feed, setSignal, type FeedQuery } from '#lib/services/projects.ts';
import { requireViewer } from '#lib/stores/authNotice.svelte.ts';

const PAGE = 20;

/** The project feed: query and loaded pages (cards carry their creator's username). */
export class FeedState {
  sort = $state<FeedSort>('trending');
  window = $state<FeedWindow>('all');
  items = $state<ProjectCard[]>([]);
  total = $state(0);
  loading = $state(true);
  error = $state<string>();

  readonly hasMore = $derived(this.items.length < this.total);

  /** Only these people's projects (comma-separated keys): a profile, or the people you follow;
   * `tag`: only items tagged with a channel or community (ADR 0068); `personal`: web's
   * personal feed (see `FeedQuery`). */
  constructor(
    readonly search: () => string,
    public creator?: string,
    public tag?: string,
    public personal?: FeedQuery['personal'],
  ) {}

  /** Counts reloads, so an answer to an earlier request is dropped. Two overlapping loads
   * (a sort change during the first load) used to append the same cards twice, and the
   * feed's keyed list then crashed the page. */
  #round = 0;

  async reload(): Promise<void> {
    this.#round++;
    this.loading = true;
    this.items = [];
    await this.more();
  }

  async more(): Promise<void> {
    const round = this.#round;
    this.loading = true;
    this.error = undefined;
    try {
      const page = await feed({
        sort: this.sort,
        window: this.window,
        q: this.search(),
        offset: this.items.length,
        ...(this.creator ? { creator: this.creator } : {}),
        ...(this.tag ? { tag: this.tag } : {}),
        ...(this.personal ? { personal: this.personal } : {}),
      });
      if (round !== this.#round) return;
      const have = this.items.map((item) => item.id);
      const fresh = page.items.slice(0, PAGE).filter((item) => !have.includes(item.id));
      this.items = [...this.items, ...fresh];
      this.total = page.total;
    } catch (e) {
      if (round === this.#round) this.error = explain(e);
    } finally {
      if (round === this.#round) this.loading = false;
    }
  }

  /** Guests see web's sign-in notice (requireViewer); the strip rolls back. */
  async signal(card: ProjectCard, next: Stance | null): Promise<void> {
    if (!requireViewer('Sign in to support this project.')) {
      throw new Error('Sign in to support this project.');
    }
    await setSignal(card.id, next);
  }
}

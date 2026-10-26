import type { FeedSort, FeedWindow, ProjectCard, Stance } from '#lib/domain/projects.ts';
import { explain } from '#lib/services/errors.ts';
import { feed, setSignal } from '#lib/services/projects.ts';
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

  /** Only these people's projects (comma-separated keys): a profile, or the people you follow. */
  constructor(
    readonly search: () => string,
    public creator?: string,
  ) {}

  async reload(): Promise<void> {
    this.loading = true;
    this.items = [];
    await this.more();
  }

  async more(): Promise<void> {
    this.loading = true;
    this.error = undefined;
    try {
      const page = await feed({
        sort: this.sort,
        window: this.window,
        q: this.search(),
        offset: this.items.length,
        ...(this.creator ? { creator: this.creator } : {}),
      });
      this.items = [...this.items, ...page.items.slice(0, PAGE)];
      this.total = page.total;
    } catch (e) {
      this.error = explain(e);
    } finally {
      this.loading = false;
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

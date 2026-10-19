import type { FeedSort, FeedWindow, ProjectCard, Stance } from '#lib/domain/projects.ts';
import { explain } from '#lib/services/errors.ts';
import { feed, setSignal } from '#lib/services/projects.ts';
import { connection } from '#lib/stores/connection.svelte.ts';

import { goto } from '$app/navigation';
import { resolve } from '$app/paths';

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

  constructor(readonly search: () => string) {}

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
      });
      this.items = [...this.items, ...page.items.slice(0, PAGE)];
      this.total = page.total;
    } catch (e) {
      this.error = explain(e);
    } finally {
      this.loading = false;
    }
  }

  /** Guests are sent to sign in (web's requireViewer); the strip rolls back. */
  async signal(card: ProjectCard, next: Stance | null): Promise<void> {
    if (connection.current.status !== 'connected') {
      await goto(resolve('/onboarding'));
      throw new Error('Sign in to support this project.');
    }
    await setSignal(card.id, next);
  }
}

import type { FeedSort, FeedWindow, ProjectCard, Stance } from '#lib/domain/projects.ts';
import { explain } from '#lib/services/errors.ts';
import { getPerson } from '#lib/services/people.ts';
import { feed, setSignal } from '#lib/services/projects.ts';

const PAGE = 20;

/** The project feed: query, loaded pages and creator names (one lookup per creator). */
export class FeedState {
  sort = $state<FeedSort>('trending');
  window = $state<FeedWindow>('all');
  items = $state<ProjectCard[]>([]);
  total = $state(0);
  loading = $state(true);
  error = $state<string>();
  names = $state<Record<string, string>>({});

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
      void this.loadNames(page.items);
    } catch (e) {
      this.error = explain(e);
    } finally {
      this.loading = false;
    }
  }

  signal(card: ProjectCard, next: Stance | null): Promise<void> {
    return setSignal(card.id, next);
  }

  private async loadNames(cards: ProjectCard[]): Promise<void> {
    const missing = cards
      .map((c) => c.creator)
      .filter((a, i, all) => all.indexOf(a) === i && !(a in this.names));
    const found = await Promise.all(
      missing.map(
        async (agent) => [agent, (await getPerson(agent)).profile?.name ?? 'Unnamed'] as const,
      ),
    );
    this.names = { ...this.names, ...Object.fromEntries(found) };
  }
}

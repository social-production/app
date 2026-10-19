import type { ThreadItem, ThreadSort, ThreadWindow, VoteDirection } from '#lib/domain/threads.ts';
import { explain } from '#lib/services/errors.ts';
import { castVote, createThread, listThreads } from '#lib/services/threads.ts';

import { viewer, withNames } from './people.ts';

/** One project's thread list (web's channel feed, threads only): sort, window, cards. */
export class ThreadsState {
  items = $state<ThreadItem[]>([]);
  sort = $state<ThreadSort>('trending');
  window = $state<ThreadWindow>('all');
  names = $state<Record<string, string>>({});
  paused = $state(false);
  loading = $state(false);
  /** Until the first load, an empty list means "still loading", not "no threads". */
  loaded = $state(false);
  error = $state('');
  creating = $state(false);
  createError = $state('');

  constructor(readonly projectId: string) {}

  async open(): Promise<void> {
    void viewer()
      .then((v) => {
        this.paused = v.paused;
        this.names = { ...this.names, [v.me]: v.name };
      })
      .catch(() => undefined);
    await this.reload();
  }

  async reload(): Promise<void> {
    this.loading = true;
    try {
      this.items = await listThreads(this.projectId, this.sort, this.window);
      this.error = '';
      this.loaded = true;
      this.names = await withNames(
        this.items.map((t) => t.author),
        this.names,
      );
    } catch (e) {
      if (!this.loaded) this.error = explain(e);
    } finally {
      this.loading = false;
    }
  }

  /** The strip has already moved; a failure throws so it rolls back. */
  async vote(item: ThreadItem, next: VoteDirection): Promise<void> {
    await castVote(this.projectId, item.id, next);
  }

  /** Write the thread; resolves to its id, or null with `createError` set. */
  async create(title: string, body: string): Promise<string | null> {
    this.creating = true;
    this.createError = '';
    try {
      const id = await createThread(this.projectId, title, body);
      void this.reload();
      return id;
    } catch (e) {
      this.createError = explain(e);
      return null;
    } finally {
      this.creating = false;
    }
  }
}

import { VISIBLE, type Moderation } from '#lib/domain/moderation.ts';
import { getModeration } from '#lib/services/moderation.ts';

/** A detail page's moderation (ADR 0071): its ⋮ menu's report, and whether it was removed. */
export class ModerationLoader {
  value = $state<Moderation>(VISIBLE);
  readonly #id: string;

  constructor(id: string) {
    this.#id = id;
  }

  async load(): Promise<void> {
    try {
      this.value = await getModeration(this.#id);
    } catch {
      // Unreadable reports leave the page as it is; the menu re-reads them when opened.
    }
  }

  set(next: Moderation): void {
    this.value = next;
  }
}

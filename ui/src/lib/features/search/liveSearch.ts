import type { SearchResultItem, SearchType } from '#lib/domain/search.ts';
import { search } from '#lib/services/search.ts';

/** web's createLiveSearchScheduler: suggestions 200 ms after typing stops; late answers lose. */
export function createLiveSearchScheduler() {
  let timer: ReturnType<typeof setTimeout> | null = null;
  let requestId = 0;

  function cancel(): void {
    if (timer) clearTimeout(timer);
    timer = null;
  }

  function schedule(
    query: string,
    type: SearchType,
    onResults: (results: SearchResultItem[], loading: boolean) => void,
  ): void {
    cancel();
    const trimmed = query.trim();
    if (!trimmed) {
      onResults([], false);
      return;
    }
    onResults([], true);
    const current = ++requestId;
    timer = setTimeout(() => {
      search(trimmed, type).then(
        (results) => {
          if (current === requestId) onResults(results, false);
        },
        () => {
          if (current === requestId) onResults([], false);
        },
      );
    }, 200);
  }

  return { schedule, cancel };
}

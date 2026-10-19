/**
 * web/src/lib/utils/visibilityPoll.ts: run `task` every `activeMs` while the page is visible and
 * `isActive()` holds, else check back every `idleMs`. Coming back to the tab runs it at once.
 * Returns a stop function.
 */
export function startVisibilityPoll(
  task: () => Promise<void> | void,
  options: { activeMs: number; idleMs?: number; isActive?: () => boolean },
): () => void {
  const idleMs = options.idleMs ?? Math.max(options.activeMs * 4, 30_000);
  const isActive = options.isActive ?? (() => true);
  let timer: number | null = null;
  let stopped = false;
  let running = false;

  const schedule = (delay: number): void => {
    if (stopped) return;
    if (timer !== null) window.clearTimeout(timer);
    timer = window.setTimeout(() => void run(), delay);
  };

  const run = async (): Promise<void> => {
    timer = null;
    if (stopped) return;
    if (document.visibilityState !== 'visible' || !isActive() || running) {
      schedule(idleMs);
      return;
    }
    running = true;
    try {
      await task();
    } finally {
      running = false;
      schedule(options.activeMs);
    }
  };

  const onVisibility = (): void => {
    if (document.visibilityState === 'visible' && isActive()) schedule(0);
  };

  document.addEventListener('visibilitychange', onVisibility);
  schedule(options.activeMs);
  return () => {
    stopped = true;
    if (timer !== null) window.clearTimeout(timer);
    document.removeEventListener('visibilitychange', onVisibility);
  };
}

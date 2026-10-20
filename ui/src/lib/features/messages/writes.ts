/**
 * One write at a time per person: two zome calls writing to the same source chain at once are
 * refused ("the source chain head has moved"). A chat's read mark and a message sent just
 * then would race, so every chat write on this page waits for the one before it.
 */
let tail: Promise<unknown> = Promise.resolve();

export function inTurn<T>(write: () => Promise<T>): Promise<T> {
  const next = tail.then(write, write);
  tail = next.catch(() => undefined);
  return next;
}

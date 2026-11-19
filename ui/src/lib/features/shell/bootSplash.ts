/**
 * The static loading cards in app.html cover the page from the first paint until the app's code
 * arrives. Once the shell mounts it shows the same cards itself (OpeningScreen) while the first
 * connection is made, so the static ones fade away at once.
 */
export function settleBootSplash(): void {
  const splash = document.getElementById('boot-splash');
  if (!splash) return;
  splash.classList.add('gone');
  setTimeout(() => splash.remove(), 250);
}

/**
 * The static opening screen in app.html (owner's fix 3c, 2026-11-17: "I have never seen the
 * breathing loading screen"): it covers the page from the first paint. While the first
 * connection is still being made it stays, and after 8 s says a slow network can take a
 * minute; once the connection settles (connected, signed out or offline) it fades away.
 */
export function settleBootSplash(status: string): (() => void) | undefined {
  const splash = document.getElementById('boot-splash');
  if (!splash) return undefined;
  if (status === 'connecting') {
    const timer = setTimeout(() => {
      const note = document.createElement('p');
      note.textContent = 'Still connecting. This can take a minute on a slow network.';
      splash.append(note);
    }, 8000);
    return () => clearTimeout(timer);
  }
  splash.classList.add('gone');
  setTimeout(() => splash.remove(), 300);
  return undefined;
}

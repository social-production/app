/**
 * web LiveChatPanel's syncPanelHeight: on phones the panel fills the space under the top bar,
 * and while the keyboard is open it is pinned above it. Returns whether the keyboard is open.
 */

const PINNED = ['position', 'left', 'right', 'top', 'bottom', 'height', 'maxHeight', 'zIndex'];

function topbarPx(): number {
  return document.querySelector<HTMLElement>('.topbar')?.getBoundingClientRect().height ?? 0;
}

function clearPinned(panel: HTMLElement): void {
  for (const key of PINNED) panel.style.setProperty(key.replace(/[A-Z]/g, '-$&').toLowerCase(), '');
}

function typing(): boolean {
  const active = document.activeElement;
  return (
    active instanceof HTMLTextAreaElement ||
    active instanceof HTMLInputElement ||
    (active instanceof HTMLElement && active.isContentEditable)
  );
}

function pinAboveKeyboard(panel: HTMLElement, viewportHeight: number, offsetTop: number): void {
  const bar = topbarPx();
  const top = Math.max(offsetTop + bar, offsetTop);
  const bottomGap = Math.max(0, window.innerHeight - offsetTop - viewportHeight);
  Object.assign(panel.style, {
    position: 'fixed',
    left: '0',
    right: '0',
    top: `${Math.floor(top)}px`,
    bottom: `${Math.floor(bottomGap)}px`,
    height: 'auto',
    maxHeight: 'none',
    zIndex: '40',
  });
  panel.style.setProperty(
    '--chat-panel-height',
    `${Math.floor(Math.max(viewportHeight - bar, 240))}px`,
  );
}

export function syncPanelHeight(
  panel: HTMLElement,
  options: { embedded: boolean; fitViewport: boolean },
): boolean {
  const vv = window.visualViewport;
  const viewportHeight = vv?.height ?? window.innerHeight;
  const offsetTop = vv?.offsetTop ?? 0;
  const keyboardOpen = Boolean(vv && window.innerHeight - vv.height > 120 && typing());

  if (options.embedded) {
    // The project page sizes the chat shell; keep the panel filling it.
    clearPinned(panel);
    return keyboardOpen;
  }
  if (!options.fitViewport) {
    panel.style.removeProperty('--chat-panel-height');
    return keyboardOpen;
  }
  if (keyboardOpen) {
    pinAboveKeyboard(panel, viewportHeight, offsetTop);
    return keyboardOpen;
  }
  clearPinned(panel);
  const topOffset = Math.max(panel.getBoundingClientRect().top, topbarPx() + 12);
  const height = Math.max(viewportHeight - topOffset, 320);
  panel.style.setProperty('--chat-panel-height', `${Math.floor(height)}px`);
  return keyboardOpen;
}

/** web's scroll to a message: centre it in the chat log. */
export function centerIn(log: HTMLElement, target: HTMLElement, behavior: ScrollBehavior): void {
  const bounds = log.getBoundingClientRect();
  const box = target.getBoundingClientRect();
  const top = box.top - bounds.top + log.scrollTop - log.clientHeight / 2 + box.height / 2;
  log.scrollTo({ top: Math.max(top, 0), behavior });
}

/** To the newest message, again after layout settles. */
export function scrollToEnd(log: HTMLElement | undefined): void {
  if (!log) return;
  const apply = (): void => {
    log.scrollTop = log.scrollHeight;
  };
  apply();
  requestAnimationFrame(apply);
}

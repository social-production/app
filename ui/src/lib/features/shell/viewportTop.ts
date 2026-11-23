/**
 * iPhone home-screen app (owner, 2026-11-24): with the keyboard open, iOS slides the whole page
 * up, so a project chat's top tabs went under the status bar and stayed there after the
 * keyboard closed, with no way to change tab. `--vv-top` is how far the visible area has slid,
 * for the tabs to stay in sight; once the keyboard closes the page is put back.
 */

function typing(): boolean {
  const el = document.activeElement;
  return (
    el instanceof HTMLInputElement ||
    el instanceof HTMLTextAreaElement ||
    (el instanceof HTMLElement && el.isContentEditable)
  );
}

/** Start following the visible area. Returns the cleanup. */
export function followViewportTop(): () => void {
  const vv = window.visualViewport;
  if (!vv) return () => {};
  const root = document.documentElement;
  let slid = false;
  const sync = (): void => {
    const top = Math.max(0, Math.round(vv.offsetTop));
    root.style.setProperty('--vv-top', `${top}px`);
    const open = innerHeight - vv.height > 120 && typing();
    if (open) slid = true;
    else if (slid) {
      // The keyboard closed: put back anything iOS left slid up.
      slid = false;
      root.style.setProperty('--vv-top', '0px');
      requestAnimationFrame(() => scrollTo({ top: scrollY }));
    }
  };
  vv.addEventListener('resize', sync);
  vv.addEventListener('scroll', sync);
  const onBlur = (): void => void setTimeout(sync, 350);
  document.addEventListener('focusout', onBlur);
  return () => {
    vv.removeEventListener('resize', sync);
    vv.removeEventListener('scroll', sync);
    document.removeEventListener('focusout', onBlur);
    root.style.removeProperty('--vv-top');
  };
}

/** Shrink a one-line title to fit its box, down to `min` px (web/src/lib/utils/fitText.ts). */
interface FitTextOptions {
  min?: number;
  text?: string;
}

export function fitText(node: HTMLElement, options: FitTextOptions = {}) {
  let frame = 0;
  let current = options;
  const observer = new ResizeObserver(() => schedule());

  function clearInline(): void {
    for (const key of [
      'display',
      'maxWidth',
      'whiteSpace',
      'overflow',
      'textOverflow',
      'fontSize',
    ] as const) {
      node.style[key] = '';
    }
  }

  function measure(): void {
    const min = current.min ?? 10;
    Object.assign(node.style, {
      display: 'block',
      maxWidth: '100%',
      whiteSpace: 'nowrap',
      overflow: 'hidden',
    });
    node.style.textOverflow = 'clip';
    node.style.fontSize = '';
    const natural = parseFloat(getComputedStyle(node).fontSize) || 13;
    const box = node.clientWidth;
    if (box <= 0) return;
    if (node.scrollWidth <= box + 1) {
      clearInline();
      return;
    }
    node.style.fontSize = `${Math.max(min, Math.floor(((natural * box) / node.scrollWidth) * 10) / 10)}px`;
    if (node.scrollWidth > box + 1) {
      Object.assign(node.style, {
        whiteSpace: 'normal',
        overflow: 'visible',
        fontSize: `${min}px`,
      });
    }
  }

  function schedule(): void {
    cancelAnimationFrame(frame);
    frame = requestAnimationFrame(measure);
  }

  if (node.parentElement) observer.observe(node.parentElement);
  schedule();

  return {
    update(next: FitTextOptions = {}) {
      current = next;
      schedule();
    },
    destroy() {
      cancelAnimationFrame(frame);
      observer.disconnect();
      clearInline();
    },
  };
}

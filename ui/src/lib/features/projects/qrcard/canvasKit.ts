/** Small canvas helpers for the QR share cards (Owner Notes, 2026-11-24): images drawn like
 * CSS `object-fit: cover`, rounded boxes and text wrapped to a width. */

export const FONT = 'Inter, system-ui, -apple-system, "Segoe UI", Roboto, sans-serif';
export const BRAND = '#2e7d5b';
export const INK = '#17181a';
export const SOFT_INK = '#4d5157';

export function font(weight: number, size: number): string {
  return `${weight} ${Math.round(size)}px ${FONT}`;
}

/** Waits for the card's faces so the first card drawn is not in a fallback font. */
export async function loadFonts(): Promise<void> {
  try {
    await Promise.all([800, 700, 600].map((w) => document.fonts.load(font(w, 40))));
  } catch {
    // Drawn in the fallback face.
  }
}

export function loadImage(src: string | null): Promise<HTMLImageElement | null> {
  if (!src) return Promise.resolve(null);
  return new Promise((resolve) => {
    const img = new Image();
    img.crossOrigin = 'anonymous';
    img.onload = () => resolve(img);
    img.onerror = () => resolve(null);
    img.src = src;
  });
}

export function roundRect(
  ctx: CanvasRenderingContext2D,
  x: number,
  y: number,
  w: number,
  h: number,
  r: number,
): void {
  ctx.beginPath();
  ctx.roundRect(x, y, w, h, r);
}

export function fillRound(
  ctx: CanvasRenderingContext2D,
  box: [number, number, number, number],
  r: number,
  color: string,
): void {
  roundRect(ctx, ...box, r);
  ctx.fillStyle = color;
  ctx.fill();
}

/** `img` filling the box as `object-fit: cover` does, clipped to radius `r`. */
export function drawCover(
  ctx: CanvasRenderingContext2D,
  img: HTMLImageElement,
  box: [number, number, number, number],
  r = 0,
): void {
  const [x, y, w, h] = box;
  const scale = Math.max(w / img.naturalWidth, h / img.naturalHeight);
  const sw = w / scale;
  const sh = h / scale;
  ctx.save();
  roundRect(ctx, x, y, w, h, r);
  ctx.clip();
  ctx.drawImage(img, (img.naturalWidth - sw) / 2, (img.naturalHeight - sh) / 2, sw, sh, x, y, w, h);
  ctx.restore();
}

/** `text` broken into at most `max` lines no wider than `width` (the last ends in "…"). */
export function wrapLines(
  ctx: CanvasRenderingContext2D,
  text: string,
  width: number,
  max: number,
): string[] {
  const words = text.replace(/\s+/g, ' ').trim().split(' ');
  const lines: string[] = [];
  let line = '';
  for (const word of words) {
    const next = line ? `${line} ${word}` : word;
    if (ctx.measureText(next).width <= width || !line) {
      line = next;
      continue;
    }
    lines.push(line);
    line = word;
    if (lines.length === max) break;
  }
  if (lines.length < max && line) lines.push(line);
  const used = lines.join(' ').split(' ').length;
  if (used < words.length && lines.length) {
    let last = lines[lines.length - 1] ?? '';
    while (last && ctx.measureText(`${last}…`).width > width) last = last.slice(0, -1);
    lines[lines.length - 1] = `${last.trimEnd()}…`;
  }
  return lines;
}

/** Draws `lines` from baseline `y`; returns the y under the last line. */
export function drawLines(
  ctx: CanvasRenderingContext2D,
  lines: string[],
  x: number,
  y: number,
  height: number,
): number {
  lines.forEach((line, i) => ctx.fillText(line, x, y + i * height));
  return y + (lines.length - 1) * height;
}

export function canvasOf(w: number, h: number): [HTMLCanvasElement, CanvasRenderingContext2D] {
  const canvas = document.createElement('canvas');
  canvas.width = w;
  canvas.height = h;
  const ctx = canvas.getContext('2d');
  if (!ctx) throw new Error('This browser cannot draw images.');
  ctx.textBaseline = 'alphabetic';
  return [canvas, ctx];
}

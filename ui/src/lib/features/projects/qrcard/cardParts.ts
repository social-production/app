import { t } from '#lib/i18n/index.ts';

import { BRAND, drawCover, fillRound, font, roundRect } from './canvasKit.ts';
import type { QrMatrix } from './qr.ts';

/** What every QR card shows of a project, service, proposal or event. */
export interface CardData {
  title: string;
  /** "#neighbourhood · Proposal". */
  kind: string;
  summary: string;
  place: string;
  url: string;
  qr: QrMatrix;
  cover: HTMLImageElement | null;
  picture: HTMLImageElement | null;
  logo: HTMLImageElement | null;
}

/** The cover, or a band of the brand's green when the space has none. */
export function drawBand(
  ctx: CanvasRenderingContext2D,
  data: CardData,
  box: [number, number, number, number],
): void {
  if (data.cover) return drawCover(ctx, data.cover, box);
  const [x, y, w, h] = box;
  const g = ctx.createLinearGradient(x, y, x + w, y + h);
  g.addColorStop(0, '#256b4c');
  g.addColorStop(1, '#4f9f78');
  ctx.fillStyle = g;
  ctx.fillRect(x, y, w, h);
}

/** The picture in a rounded frame of `ring` in `ringColor`; the logo tile when it has none. */
export function drawPicture(
  ctx: CanvasRenderingContext2D,
  data: CardData,
  box: [number, number, number],
  ring: number,
  ringColor: string,
): void {
  const [x, y, side] = box;
  const r = side * 0.24;
  fillRound(ctx, [x, y, side, side], r, ringColor);
  const inner: [number, number, number, number] = [
    x + ring,
    y + ring,
    side - ring * 2,
    side - ring * 2,
  ];
  if (data.picture) return drawCover(ctx, data.picture, inner, r - ring);
  fillRound(ctx, inner, r - ring, BRAND);
  if (data.logo)
    ctx.drawImage(
      data.logo,
      inner[0] + inner[2] * 0.2,
      inner[1] + inner[3] * 0.2,
      inner[2] * 0.6,
      inner[3] * 0.6,
    );
}

/** The logo (in a green tile unless `bare`) and "Social Production", from (x, middle y). */
export function drawBrandRow(
  ctx: CanvasRenderingContext2D,
  data: CardData,
  at: [number, number],
  size: number,
  color: string,
  bare = false,
): void {
  const [x, y] = at;
  if (!bare) {
    roundRect(ctx, x, y - size / 2, size, size, size * 0.28);
    ctx.fillStyle = BRAND;
    ctx.fill();
  }
  const pad = bare ? 0 : size * 0.16;
  if (data.logo)
    ctx.drawImage(data.logo, x + pad, y - size / 2 + pad, size - pad * 2, size - pad * 2);
  ctx.fillStyle = color;
  ctx.font = font(700, size * 0.46);
  ctx.textBaseline = 'middle';
  ctx.fillText('Social Production', x + size + size * 0.25, y);
  ctx.textBaseline = 'alphabetic';
}

/** Drawn on every card, in the viewer's language. */
export const scanLine = (): string => t('Scan to see it and join');

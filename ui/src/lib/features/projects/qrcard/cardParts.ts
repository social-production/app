import { t } from '#lib/i18n/index.ts';

import { BRAND, INK, SOFT_INK, drawCover, fillRound, font, roundRect } from './canvasKit.ts';
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
  picture: HTMLImageElement | null;
  logo: HTMLImageElement | null;
}

/** A card's colours: white or dark paper (owner, 2026-11-24: no cover photo behind the text,
 * the picture is enough). */
export interface Paper {
  bg: string;
  ink: string;
  soft: string;
  body: string;
  /** "Scan to …" and other green words. */
  accent: string;
  /** The tinted box behind a flyer's code, and the QR panel's edge on white. */
  tint: string;
  edge: string | null;
}

export const WHITE: Paper = {
  bg: '#ffffff',
  ink: INK,
  soft: SOFT_INK,
  body: '#3a3d42',
  accent: BRAND,
  tint: '#eef5f1',
  edge: '#dfe4e0',
};

export const DARK: Paper = {
  bg: '#16181b',
  ink: '#f4f5f3',
  soft: '#a7adb3',
  body: '#d3d7da',
  accent: '#7fd3a8',
  tint: '#22302a',
  edge: null,
};

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

import { t } from '#lib/i18n/index.ts';

import { BRAND, canvasOf, drawLines, font, roundRect, wrapLines } from './canvasKit.ts';
import { drawBrandRow, type CardData } from './cardParts.ts';
import { MM } from './cardsPrint.ts';
import { drawQrPanel } from './qr.ts';

/** Stickers on the brand's green with 3 mm of bleed and a dashed cut line: round, 60 mm across
 * (a 66 mm page), or square, 50 mm with rounded corners (a 56 mm page), mostly code (owner,
 * 2026-11-24). */
export type StickerShape = 'round' | 'square';

export const stickerPage = (shape: StickerShape): number => (shape === 'round' ? 66 : 56);

export function stickerCard(data: CardData, shape: StickerShape): HTMLCanvasElement {
  const side = Math.round(stickerPage(shape) * MM);
  const [canvas, ctx] = canvasOf(side, side);
  const c = side / 2;
  ctx.fillStyle = '#ffffff';
  ctx.fillRect(0, 0, side, side);
  if (shape === 'round') {
    ctx.beginPath();
    ctx.arc(c, c, side / 2, 0, Math.PI * 2);
  } else ctx.rect(0, 0, side, side);
  ctx.fillStyle = BRAND;
  ctx.fill();
  if (shape === 'round') {
    ctx.beginPath();
    ctx.arc(c, c, 30 * MM, 0, Math.PI * 2);
  } else roundRect(ctx, 3 * MM, 3 * MM, side - 6 * MM, side - 6 * MM, 4 * MM);
  ctx.setLineDash([12, 10]);
  ctx.lineWidth = 2;
  ctx.strokeStyle = 'rgba(255,255,255,0.7)';
  ctx.stroke();
  ctx.setLineDash([]);
  if (shape === 'round') roundFace(ctx, data, c);
  else squareFace(ctx, data, c);
  return canvas;
}

function roundFace(ctx: CanvasRenderingContext2D, data: CardData, c: number): void {
  ctx.textAlign = 'center';
  ctx.fillStyle = '#ffffff';
  ctx.font = font(800, 2.6 * MM);
  ctx.fillText(t('Scan to join').toUpperCase(), c, c - 13.5 * MM);
  const qr = 24 * MM;
  drawQrPanel(ctx, data.qr, [c - qr / 2, c - 11.5 * MM, qr], data.logo, 1.9 * MM, 3 * MM);
  ctx.fillStyle = '#ffffff';
  ctx.font = font(800, 3.6 * MM);
  drawLines(ctx, wrapLines(ctx, data.title, 34 * MM, 2), c, c + 17 * MM, 3.9 * MM);
  brand(ctx, data, c, c + 25.5 * MM);
}

/** The code fills most of the square; "Scan to join" above it, the title and logo under. */
function squareFace(ctx: CanvasRenderingContext2D, data: CardData, c: number): void {
  ctx.textAlign = 'center';
  ctx.fillStyle = '#ffffff';
  ctx.font = font(800, 2.5 * MM);
  ctx.fillText(t('Scan to join').toUpperCase(), c, 8.6 * MM);
  const qr = 29 * MM;
  drawQrPanel(ctx, data.qr, [c - qr / 2, 10.4 * MM, qr], data.logo, 2 * MM, 3 * MM);
  ctx.fillStyle = '#ffffff';
  ctx.font = font(800, 3.3 * MM);
  drawLines(ctx, wrapLines(ctx, data.title, 42 * MM, 2), c, 43.6 * MM, 3.6 * MM);
  brand(ctx, data, c, 49.6 * MM);
}

function brand(ctx: CanvasRenderingContext2D, data: CardData, c: number, y: number): void {
  ctx.textAlign = 'left';
  const size = 3 * MM;
  ctx.font = font(700, size * 0.46);
  const label = ctx.measureText('Social Production').width;
  drawBrandRow(ctx, data, [c - (size * 1.25 + label) / 2, y], size, '#ffffff', true);
}

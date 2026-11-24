import { t } from '#lib/i18n/index.ts';

import { canvasOf, drawLines, font, wrapLines } from './canvasKit.ts';
import { scanLine, drawBrandRow, drawPicture, type CardData, type Paper } from './cardParts.ts';
import { drawQr, drawQrPanel } from './qr.ts';

/** Print sizes at 300 dpi. */
export const MM = 300 / 25.4;

/** One A6 flyer, 105 × 148 mm. */
export function flyerCard(data: CardData, paper: Paper): HTMLCanvasElement {
  const [w, h] = [Math.round(105 * MM), Math.round(148 * MM)];
  const [canvas, ctx] = canvasOf(w, h);
  const k = w / 840;
  ctx.fillStyle = paper.bg;
  ctx.fillRect(0, 0, w, h);
  drawPicture(ctx, data, [64 * k, 64 * k, 140 * k], 0, paper.bg);
  ctx.fillStyle = paper.soft;
  ctx.font = font(600, 26 * k);
  ctx.fillText(wrapLines(ctx, data.kind, 520 * k, 1)[0] ?? '', 232 * k, 144 * k);
  ctx.fillStyle = paper.ink;
  ctx.font = font(800, 72 * k);
  let y = drawLines(ctx, wrapLines(ctx, data.title, 712 * k, 3), 64 * k, 320 * k, 78 * k);
  ctx.fillStyle = paper.body;
  ctx.font = font(400, 30 * k);
  y = drawLines(ctx, wrapLines(ctx, data.summary, 712 * k, 5), 64 * k, y + 64 * k, 42 * k);
  if (data.place) {
    ctx.font = font(600, 26 * k);
    ctx.fillText(wrapLines(ctx, data.place, 712 * k, 1)[0] ?? '', 64 * k, y + 50 * k);
  }
  flyerBand(ctx, data, paper, [k, h]);
  return canvas;
}

function flyerBand(
  ctx: CanvasRenderingContext2D,
  data: CardData,
  paper: Paper,
  [k, h]: [number, number],
): void {
  const top = h - 56 * k - 344 * k;
  ctx.beginPath();
  ctx.roundRect(64 * k, top, 712 * k, 344 * k, 32 * k);
  ctx.fillStyle = paper.tint;
  ctx.fill();
  drawQrPanel(ctx, data.qr, [96 * k, top + 32 * k, 280 * k], data.logo, 20 * k, 24 * k);
  const x = 410 * k;
  ctx.fillStyle = paper.accent;
  ctx.font = font(800, 38 * k);
  const y = drawLines(ctx, wrapLines(ctx, scanLine(), 340 * k, 2), x, top + 90 * k, 44 * k);
  ctx.fillStyle = paper.body;
  ctx.font = font(400, 23 * k);
  const lines = wrapLines(
    ctx,
    t('Support it, ask questions and help plan it. Free, and run by its members.'),
    340 * k,
    4,
  );
  drawLines(ctx, lines, x, y + 46 * k, 32 * k);
  drawBrandRow(ctx, data, [x, top + 344 * k - 58 * k], 52 * k, paper.ink);
}

/** Four A6 flyers on an A4 page with crop marks at their corners. */
export function flyerSheet(data: CardData, paper: Paper): HTMLCanvasElement {
  const [w, h] = [Math.round(210 * MM), Math.round(297 * MM)];
  const [canvas, ctx] = canvasOf(w, h);
  ctx.fillStyle = '#ffffff';
  ctx.fillRect(0, 0, w, h);
  const one = flyerCard(data, paper);
  const fw = Math.round(105 * MM);
  const fh = Math.round(148 * MM);
  const top = Math.round((h - fh * 2) / 2);
  for (const [x, y] of [
    [0, top],
    [fw, top],
    [0, top + fh],
    [fw, top + fh],
  ] as const)
    ctx.drawImage(one, x, y, fw, fh);
  ctx.strokeStyle = '#9aa19b';
  ctx.lineWidth = 2;
  for (const y of [top, top + fh, top + fh * 2]) {
    ctx.beginPath();
    ctx.moveTo(0, y);
    ctx.lineTo(40, y);
    ctx.moveTo(w - 40, y);
    ctx.lineTo(w, y);
    ctx.stroke();
  }
  ctx.beginPath();
  ctx.moveTo(fw, top - 40);
  ctx.lineTo(fw, top - 8);
  ctx.moveTo(fw, top + fh * 2 + 8);
  ctx.lineTo(fw, top + fh * 2 + 40);
  ctx.stroke();
  return canvas;
}

/** Just the code on white with a quiet margin, 2048 px. */
export function plainQr(data: CardData): HTMLCanvasElement {
  const [canvas, ctx] = canvasOf(2048, 2048);
  ctx.fillStyle = '#ffffff';
  ctx.fillRect(0, 0, 2048, 2048);
  drawQr(ctx, data.qr, [160, 160, 1728], data.logo);
  return canvas;
}

/** The code with the title and the logo under it, 1200 × 1500. */
export function labelQr(data: CardData, paper: Paper): HTMLCanvasElement {
  const [canvas, ctx] = canvasOf(1200, 1500);
  ctx.fillStyle = paper.bg;
  ctx.fillRect(0, 0, 1200, 1500);
  if (paper.edge) drawQr(ctx, data.qr, [100, 100, 1000], data.logo);
  else drawQrPanel(ctx, data.qr, [70, 70, 1060], data.logo, 30, 48);
  ctx.textAlign = 'center';
  ctx.fillStyle = paper.ink;
  ctx.font = font(800, 76);
  drawLines(ctx, wrapLines(ctx, data.title, 1040, 2), 600, 1230, 84);
  ctx.textAlign = 'left';
  ctx.font = font(700, 40);
  const label = ctx.measureText('Social Production').width;
  drawBrandRow(ctx, data, [600 - (86 * 1.25 + label) / 2, 1420], 86, paper.accent);
  return canvas;
}

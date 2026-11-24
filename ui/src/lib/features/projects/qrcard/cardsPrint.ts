import { t } from '#lib/i18n/index.ts';

import { BRAND, INK, SOFT_INK, canvasOf, drawLines, font, wrapLines } from './canvasKit.ts';
import { scanLine, drawBand, drawBrandRow, drawPicture, type CardData } from './cardParts.ts';
import { drawQr, drawQrPanel } from './qr.ts';

/** Print sizes at 300 dpi. */
export const MM = 300 / 25.4;

/** The round sticker: 60 mm across with 3 mm of bleed, the cut line dashed (66 mm page). */
export function stickerCard(data: CardData): HTMLCanvasElement {
  const side = Math.round(66 * MM);
  const [canvas, ctx] = canvasOf(side, side);
  const c = side / 2;
  ctx.fillStyle = '#ffffff';
  ctx.fillRect(0, 0, side, side);
  ctx.beginPath();
  ctx.arc(c, c, side / 2, 0, Math.PI * 2);
  ctx.fillStyle = BRAND;
  ctx.fill();
  ctx.beginPath();
  ctx.arc(c, c, 30 * MM, 0, Math.PI * 2);
  ctx.setLineDash([12, 10]);
  ctx.lineWidth = 2;
  ctx.strokeStyle = 'rgba(255,255,255,0.7)';
  ctx.stroke();
  ctx.setLineDash([]);

  ctx.textAlign = 'center';
  ctx.fillStyle = '#ffffff';
  ctx.font = font(800, 2.6 * MM);
  ctx.fillText(t('Scan to join').toUpperCase(), c, c - 13.5 * MM);
  const qr = 24 * MM;
  drawQrPanel(ctx, data.qr, [c - qr / 2, c - 11.5 * MM, qr], data.logo, 1.9 * MM, 3 * MM);
  ctx.fillStyle = '#ffffff';
  ctx.font = font(800, 3.6 * MM);
  drawLines(ctx, wrapLines(ctx, data.title, 34 * MM, 2), c, c + 17 * MM, 3.9 * MM);
  ctx.textAlign = 'left';
  ctx.font = font(700, 1.9 * MM);
  const label = ctx.measureText('Social Production').width;
  const size = 3 * MM;
  drawBrandRow(ctx, data, [c - (size * 1.25 + label) / 2, c + 25.5 * MM], size, '#ffffff', true);
  return canvas;
}

/** One A6 flyer, 105 × 148 mm. */
export function flyerCard(data: CardData): HTMLCanvasElement {
  const [w, h] = [Math.round(105 * MM), Math.round(148 * MM)];
  const [canvas, ctx] = canvasOf(w, h);
  const k = w / 840;
  ctx.fillStyle = '#ffffff';
  ctx.fillRect(0, 0, w, h);
  drawBand(ctx, data, [0, 0, w, 280 * k]);
  drawPicture(ctx, data, [64 * k, 220 * k, 140 * k], 8 * k, '#ffffff');
  ctx.fillStyle = SOFT_INK;
  ctx.font = font(600, 26 * k);
  ctx.fillText(data.kind, 228 * k, 344 * k);
  ctx.fillStyle = INK;
  ctx.font = font(800, 72 * k);
  let y = drawLines(ctx, wrapLines(ctx, data.title, 712 * k, 2), 64 * k, 460 * k, 76 * k);
  ctx.fillStyle = '#3a3d42';
  ctx.font = font(400, 30 * k);
  y = drawLines(ctx, wrapLines(ctx, data.summary, 712 * k, 4), 64 * k, y + 58 * k, 42 * k);
  if (data.place) {
    ctx.font = font(600, 26 * k);
    ctx.fillText(wrapLines(ctx, data.place, 712 * k, 1)[0] ?? '', 64 * k, y + 50 * k);
  }
  flyerBand(ctx, data, k, h);
  return canvas;
}

function flyerBand(ctx: CanvasRenderingContext2D, data: CardData, k: number, h: number): void {
  const top = h - 56 * k - 344 * k;
  ctx.beginPath();
  ctx.roundRect(64 * k, top, 712 * k, 344 * k, 32 * k);
  ctx.fillStyle = '#eef5f1';
  ctx.fill();
  drawQrPanel(ctx, data.qr, [96 * k, top + 32 * k, 280 * k], data.logo, 20 * k, 24 * k);
  const x = 410 * k;
  ctx.fillStyle = BRAND;
  ctx.font = font(800, 38 * k);
  const y = drawLines(ctx, wrapLines(ctx, scanLine(), 340 * k, 2), x, top + 90 * k, 44 * k);
  ctx.fillStyle = '#3a3d42';
  ctx.font = font(400, 23 * k);
  const lines = wrapLines(
    ctx,
    t('Support it, ask questions and help plan it. Free, and run by its members.'),
    340 * k,
    4,
  );
  drawLines(ctx, lines, x, y + 46 * k, 32 * k);
  drawBrandRow(ctx, data, [x, top + 344 * k - 58 * k], 52 * k, INK);
}

/** Four A6 flyers on an A4 page with crop marks at their corners. */
export function flyerSheet(data: CardData): HTMLCanvasElement {
  const [w, h] = [Math.round(210 * MM), Math.round(297 * MM)];
  const [canvas, ctx] = canvasOf(w, h);
  ctx.fillStyle = '#ffffff';
  ctx.fillRect(0, 0, w, h);
  const one = flyerCard(data);
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
export function labelQr(data: CardData): HTMLCanvasElement {
  const [canvas, ctx] = canvasOf(1200, 1500);
  ctx.fillStyle = '#ffffff';
  ctx.fillRect(0, 0, 1200, 1500);
  drawQr(ctx, data.qr, [100, 100, 1000], data.logo);
  ctx.textAlign = 'center';
  ctx.fillStyle = INK;
  ctx.font = font(800, 76);
  drawLines(ctx, wrapLines(ctx, data.title, 1040, 2), 600, 1230, 84);
  ctx.textAlign = 'left';
  ctx.font = font(700, 40);
  const label = ctx.measureText('Social Production').width;
  drawBrandRow(ctx, data, [600 - (86 * 1.25 + label) / 2, 1420], 86, BRAND);
  return canvas;
}

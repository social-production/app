import { BRAND, INK, SOFT_INK, canvasOf, drawLines, font, wrapLines } from './canvasKit.ts';
import { scanLine, drawBand, drawBrandRow, drawPicture, type CardData } from './cardParts.ts';
import { drawQrPanel } from './qr.ts';

/** The square post, 1080 × 1080 (QR Share Cards mockup). */
export function squareCard(data: CardData): HTMLCanvasElement {
  const [canvas, ctx] = canvasOf(1080, 1080);
  const paper = '#f6f3ec';
  ctx.fillStyle = paper;
  ctx.fillRect(0, 0, 1080, 1080);
  drawBand(ctx, data, [0, 0, 1080, 300]);
  drawPicture(ctx, data, [72, 216, 168], 10, paper);

  const width = 1080 - 72 - 56 - 380 - 72;
  ctx.fillStyle = SOFT_INK;
  ctx.font = font(600, 28);
  ctx.fillText(data.kind, 72, 440);
  ctx.fillStyle = INK;
  ctx.font = font(800, 80);
  const y = drawLines(ctx, wrapLines(ctx, data.title, width, 3), 72, 530, 84);
  ctx.fillStyle = '#3a3d42';
  ctx.font = font(400, 30);
  drawLines(ctx, wrapLines(ctx, data.summary, width, 4), 72, y + 62, 41);

  drawBrandRow(ctx, data, [72, 1080 - 64 - 36], 72, INK);
  drawQrPanel(ctx, data.qr, [628, 584, 380], data.logo, 30, 40);
  ctx.fillStyle = BRAND;
  ctx.font = font(700, 30);
  ctx.textAlign = 'center';
  ctx.fillText(scanLine(), 628 + 190, 1080 - 64 - 8);
  ctx.textAlign = 'left';
  return canvas;
}

/** The story, 1080 × 1920, on the brand's green. */
export function storyCard(data: CardData): HTMLCanvasElement {
  const [canvas, ctx] = canvasOf(1080, 1920);
  ctx.fillStyle = BRAND;
  ctx.fillRect(0, 0, 1080, 1920);
  drawBand(ctx, data, [0, 0, 1080, 620]);
  const fade = ctx.createLinearGradient(0, 400, 0, 620);
  fade.addColorStop(0, 'rgba(46,125,91,0)');
  fade.addColorStop(1, BRAND);
  ctx.fillStyle = fade;
  ctx.fillRect(0, 400, 1080, 220);
  drawPicture(ctx, data, [430, 470, 220], 12, BRAND);

  ctx.textAlign = 'center';
  ctx.fillStyle = 'rgba(255,255,255,0.85)';
  ctx.font = font(600, 38);
  ctx.fillText(data.kind, 540, 760);
  ctx.fillStyle = '#ffffff';
  ctx.font = font(800, 104);
  let y = drawLines(ctx, wrapLines(ctx, data.title, 900, 3), 540, 880, 108);
  ctx.fillStyle = 'rgba(255,255,255,0.9)';
  ctx.font = font(400, 40);
  y = drawLines(ctx, wrapLines(ctx, data.summary, 840, 3), 540, y + 76, 54);

  const qrTop = Math.max(y + 60, 1180);
  drawQrPanel(ctx, data.qr, [290, qrTop, 500], data.logo, 40, 52);
  ctx.fillStyle = '#ffffff';
  ctx.font = font(700, 40);
  ctx.fillText(scanLine(), 540, qrTop + 500 + 70);
  ctx.textAlign = 'left';
  ctx.font = font(700, 36);
  const label = ctx.measureText('Social Production').width;
  drawBrandRow(ctx, data, [540 - (64 + 16 + label) / 2, 1920 - 100], 64, '#ffffff', true);
  return canvas;
}

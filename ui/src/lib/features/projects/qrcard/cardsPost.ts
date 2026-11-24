import { canvasOf, drawLines, font, wrapLines } from './canvasKit.ts';
import { scanLine, drawBrandRow, drawPicture, type CardData, type Paper } from './cardParts.ts';
import { drawQrPanel } from './qr.ts';

/** The square post, 1080 × 1080, on white or dark paper: the picture, then the words, the code
 * bottom right (owner, 2026-11-24: no cover photo behind the text). */
export function squareCard(data: CardData, paper: Paper): HTMLCanvasElement {
  const [canvas, ctx] = canvasOf(1080, 1080);
  ctx.fillStyle = paper.bg;
  ctx.fillRect(0, 0, 1080, 1080);
  drawPicture(ctx, data, [72, 72, 168], 0, paper.bg);

  const width = 1080 - 72 * 2;
  ctx.fillStyle = paper.soft;
  ctx.font = font(600, 30);
  ctx.fillText(wrapLines(ctx, data.kind, 1080 - 72 - 270, 1)[0] ?? '', 270, 168);
  ctx.fillStyle = paper.ink;
  ctx.font = font(800, 80);
  const y = drawLines(ctx, wrapLines(ctx, data.title, width, 2), 72, 350, 86);
  ctx.fillStyle = paper.body;
  ctx.font = font(400, 30);
  const summaryWidth = 1080 - 72 - 56 - 380 - 72;
  drawLines(ctx, wrapLines(ctx, data.summary, summaryWidth, 7), 72, Math.max(y + 70, 470), 42);

  drawBrandRow(ctx, data, [72, 1080 - 64 - 36], 72, paper.ink);
  drawQrPanel(ctx, data.qr, [628, 584, 380], data.logo, 30, 40, paper.edge);
  ctx.fillStyle = paper.accent;
  ctx.font = font(700, 30);
  ctx.textAlign = 'center';
  ctx.fillText(scanLine(), 628 + 190, 1080 - 64 - 8);
  ctx.textAlign = 'left';
  return canvas;
}

/** The story, 1080 × 1920, centred on white or dark paper. */
export function storyCard(data: CardData, paper: Paper): HTMLCanvasElement {
  const [canvas, ctx] = canvasOf(1080, 1920);
  ctx.fillStyle = paper.bg;
  ctx.fillRect(0, 0, 1080, 1920);
  drawPicture(ctx, data, [420, 200, 240], 0, paper.bg);

  ctx.textAlign = 'center';
  ctx.fillStyle = paper.soft;
  ctx.font = font(600, 38);
  ctx.fillText(wrapLines(ctx, data.kind, 900, 1)[0] ?? '', 540, 540);
  ctx.fillStyle = paper.ink;
  ctx.font = font(800, 104);
  let y = drawLines(ctx, wrapLines(ctx, data.title, 900, 3), 540, 670, 110);
  ctx.fillStyle = paper.body;
  ctx.font = font(400, 40);
  y = drawLines(ctx, wrapLines(ctx, data.summary, 840, 3), 540, y + 80, 54);

  const qrTop = Math.max(y + 80, 1140);
  drawQrPanel(ctx, data.qr, [270, qrTop, 540], data.logo, 40, 52, paper.edge);
  ctx.fillStyle = paper.accent;
  ctx.font = font(700, 40);
  ctx.fillText(scanLine(), 540, qrTop + 540 + 70);
  ctx.textAlign = 'left';
  ctx.font = font(700, 64 * 0.46);
  const label = ctx.measureText('Social Production').width;
  drawBrandRow(ctx, data, [540 - (64 * 1.25 + label) / 2, 1920 - 90], 64, paper.ink);
  return canvas;
}

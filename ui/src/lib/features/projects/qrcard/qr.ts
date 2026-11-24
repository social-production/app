import qrcode from 'qrcode-generator';

import { BRAND, roundRect } from './canvasKit.ts';

/**
 * A page's QR code with the app's logo over its middle. High error correction (H, 30%) keeps it
 * scanning under a logo tile about a fifth of its width.
 */
export interface QrMatrix {
  size: number;
  dark: (row: number, col: number) => boolean;
}

export function qrMatrix(text: string): QrMatrix {
  const code = qrcode(0, 'H');
  code.addData(text);
  code.make();
  return { size: code.getModuleCount(), dark: (r, c) => code.isDark(r, c) };
}

/** The logo tile's side as a share of the code's width. */
const LOGO = 0.21;

/** Draws the modules in the box (no white margin; callers draw their own panel). */
export function drawQr(
  ctx: CanvasRenderingContext2D,
  qr: QrMatrix,
  box: [number, number, number],
  logo: HTMLImageElement | null,
): void {
  const [x, y, side] = box;
  const cell = side / qr.size;
  ctx.fillStyle = '#17181a';
  for (let r = 0; r < qr.size; r++)
    for (let c = 0; c < qr.size; c++)
      if (qr.dark(r, c)) ctx.fillRect(x + c * cell, y + r * cell, Math.ceil(cell), Math.ceil(cell));
  const tile = side * LOGO;
  const ring = tile * 0.12;
  const tx = x + (side - tile) / 2;
  const ty = y + (side - tile) / 2;
  roundRect(ctx, tx - ring, ty - ring, tile + ring * 2, tile + ring * 2, tile * 0.32);
  ctx.fillStyle = '#ffffff';
  ctx.fill();
  roundRect(ctx, tx, ty, tile, tile, tile * 0.26);
  ctx.fillStyle = BRAND;
  ctx.fill();
  if (logo) ctx.drawImage(logo, tx + tile * 0.19, ty + tile * 0.19, tile * 0.62, tile * 0.62);
}

/** The same code as an SVG with a quiet margin, for people's own designs. */
export function qrSvg(qr: QrMatrix, logoDataUrl: string | null): string {
  const margin = 4;
  const n = qr.size + margin * 2;
  let path = '';
  for (let r = 0; r < qr.size; r++)
    for (let c = 0; c < qr.size; c++)
      if (qr.dark(r, c)) path += `M${c + margin} ${r + margin}h1v1h-1z`;
  const tile = qr.size * LOGO;
  const ring = tile * 0.12;
  const t = margin + (qr.size - tile) / 2;
  const logo = logoDataUrl
    ? `<image href="${logoDataUrl}" x="${t + tile * 0.19}" y="${t + tile * 0.19}" width="${tile * 0.62}" height="${tile * 0.62}"/>`
    : '';
  return [
    `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 ${n} ${n}" shape-rendering="crispEdges">`,
    `<rect width="${n}" height="${n}" fill="#ffffff"/>`,
    `<path d="${path}" fill="#17181a"/>`,
    `<rect x="${t - ring}" y="${t - ring}" width="${tile + ring * 2}" height="${tile + ring * 2}" rx="${tile * 0.32}" fill="#ffffff" shape-rendering="auto"/>`,
    `<rect x="${t}" y="${t}" width="${tile}" height="${tile}" rx="${tile * 0.26}" fill="${BRAND}" shape-rendering="auto"/>`,
    logo,
    '</svg>',
  ].join('');
}

/** A white rounded panel holding the code, `pad` inside its edge. */
export function drawQrPanel(
  ctx: CanvasRenderingContext2D,
  qr: QrMatrix,
  box: [number, number, number],
  logo: HTMLImageElement | null,
  pad: number,
  radius: number,
): void {
  const [x, y, side] = box;
  roundRect(ctx, x, y, side, side, radius);
  ctx.fillStyle = '#ffffff';
  ctx.fill();
  drawQr(ctx, qr, [x + pad, y + pad, side - pad * 2], logo);
}

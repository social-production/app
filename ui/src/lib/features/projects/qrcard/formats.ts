import type { CardData } from './cardParts.ts';
import { squareCard, storyCard } from './cardsPost.ts';
import { flyerCard, flyerSheet, labelQr, plainQr, stickerCard } from './cardsPrint.ts';
import { jpegPdf } from './pdf.ts';
import { qrSvg } from './qr.ts';

/** The QR card formats the Links tab offers (QR Share Cards mockup, owner 2026-11-24). */
export type CardFormat = 'square' | 'story' | 'sticker' | 'flyer' | 'qr' | 'qr-title';

export interface FormatInfo {
  id: CardFormat;
  label: string;
  detail: string;
  /** What Download saves. */
  file: 'PNG' | 'PDF' | 'SVG';
}

export const FORMATS: FormatInfo[] = [
  { id: 'square', label: 'Square post', detail: 'PNG · 1080 × 1080', file: 'PNG' },
  { id: 'story', label: 'Story', detail: 'PNG · 1080 × 1920', file: 'PNG' },
  { id: 'sticker', label: 'Round sticker', detail: 'PDF · 60 mm, cut line', file: 'PDF' },
  { id: 'flyer', label: 'A6 flyer', detail: 'PDF · four on A4', file: 'PDF' },
  { id: 'qr', label: 'QR code only', detail: 'PNG · or SVG', file: 'PNG' },
  { id: 'qr-title', label: 'QR with title', detail: 'PNG · for slides, menus', file: 'PNG' },
];

/** The format drawn (for the preview, a flyer is one A6, not the A4 sheet). */
export function drawFormat(format: CardFormat, data: CardData, forFile = false): HTMLCanvasElement {
  switch (format) {
    case 'square':
      return squareCard(data);
    case 'story':
      return storyCard(data);
    case 'sticker':
      return stickerCard(data);
    case 'flyer':
      // The preview is one flyer; the file is the A4 sheet of four.
      return forFile ? flyerSheet(data) : flyerCard(data);
    case 'qr':
      return plainQr(data);
    case 'qr-title':
      return labelQr(data);
  }
}

const blobOf = (canvas: HTMLCanvasElement, type: string, quality?: number): Promise<Blob> =>
  new Promise((resolve, reject) =>
    canvas.toBlob(
      (b) => (b ? resolve(b) : reject(new Error('Could not make the image.'))),
      type,
      quality,
    ),
  );

/** The file Download saves, with its name. */
export async function cardFile(
  format: CardFormat,
  data: CardData,
  svg = false,
): Promise<{ blob: Blob; name: string }> {
  const slug =
    data.title
      .normalize('NFD')
      .replace(/[̀-ͯ]/g, '')
      .toLowerCase()
      .replace(/[^a-z0-9]+/g, '-')
      .replace(/^-|-$/g, '') || 'project';
  if (format === 'qr' && svg) {
    const logo = data.logo ? logoDataUrl(data.logo) : null;
    return {
      blob: new Blob([qrSvg(data.qr, logo)], { type: 'image/svg+xml' }),
      name: `${slug}-qr.svg`,
    };
  }
  const canvas = drawFormat(format, data, true);
  const info = FORMATS.find((f) => f.id === format);
  if (info?.file === 'PDF') {
    const jpeg = new Uint8Array(await (await blobOf(canvas, 'image/jpeg', 0.95)).arrayBuffer());
    const mm = format === 'sticker' ? [66, 66] : [210, 297];
    return { blob: jpegPdf(jpeg, canvas.width, canvas.height, mm), name: `${slug}-${format}.pdf` };
  }
  return { blob: await blobOf(canvas, 'image/png'), name: `${slug}-${format}.png` };
}

function logoDataUrl(img: HTMLImageElement): string {
  const c = document.createElement('canvas');
  c.width = 256;
  c.height = 256;
  c.getContext('2d')?.drawImage(img, 0, 0, 256, 256);
  return c.toDataURL('image/png');
}

import { DARK, WHITE, type CardData } from './cardParts.ts';
import { squareCard, storyCard } from './cardsPost.ts';
import { flyerCard, flyerSheet, labelQr, plainQr } from './cardsPrint.ts';
import { stickerCard, stickerPage, type StickerShape } from './cardsSticker.ts';
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
  /** Offers white or dark paper. */
  papers?: boolean;
}

/** The sheet's choices besides the format. */
export interface CardStyle {
  dark: boolean;
  shape: StickerShape;
}

export const FORMATS: FormatInfo[] = [
  { id: 'square', label: 'Square post', detail: 'PNG · 1080 × 1080', file: 'PNG', papers: true },
  { id: 'story', label: 'Story', detail: 'PNG · 1080 × 1920', file: 'PNG', papers: true },
  { id: 'sticker', label: 'Sticker', detail: 'PDF · round or square', file: 'PDF' },
  { id: 'flyer', label: 'A6 flyer', detail: 'PDF · four on A4', file: 'PDF', papers: true },
  { id: 'qr', label: 'QR code only', detail: 'PNG · or SVG', file: 'PNG' },
  {
    id: 'qr-title',
    label: 'QR with title',
    detail: 'PNG · for slides, menus',
    file: 'PNG',
    papers: true,
  },
];

/** The format drawn (for the preview, a flyer is one A6, not the A4 sheet). */
export function drawFormat(
  format: CardFormat,
  data: CardData,
  style: CardStyle,
  forFile = false,
): HTMLCanvasElement {
  const paper = style.dark ? DARK : WHITE;
  switch (format) {
    case 'square':
      return squareCard(data, paper);
    case 'story':
      return storyCard(data, paper);
    case 'sticker':
      return stickerCard(data, style.shape);
    case 'flyer':
      // The preview is one flyer; the file is the A4 sheet of four.
      return forFile ? flyerSheet(data, paper) : flyerCard(data, paper);
    case 'qr':
      return plainQr(data);
    case 'qr-title':
      return labelQr(data, paper);
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
  style: CardStyle,
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
  const canvas = drawFormat(format, data, style, true);
  const info = FORMATS.find((f) => f.id === format);
  if (info?.file === 'PDF') {
    const jpeg = new Uint8Array(await (await blobOf(canvas, 'image/jpeg', 0.95)).arrayBuffer());
    const page = stickerPage(style.shape);
    const mm = format === 'sticker' ? [page, page] : [210, 297];
    const name = format === 'sticker' ? `${slug}-sticker-${style.shape}` : `${slug}-${format}`;
    return { blob: jpegPdf(jpeg, canvas.width, canvas.height, mm), name: `${name}.pdf` };
  }
  return {
    blob: await blobOf(canvas, 'image/png'),
    name: `${slug}-${format}${info?.papers && style.dark ? '-dark' : ''}.png`,
  };
}

function logoDataUrl(img: HTMLImageElement): string {
  const c = document.createElement('canvas');
  c.width = 256;
  c.height = 256;
  c.getContext('2d')?.drawImage(img, 0, 0, 256, 256);
  return c.toDataURL('image/png');
}

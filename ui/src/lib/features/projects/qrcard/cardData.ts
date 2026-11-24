import brandLogo from '#lib/assets/brand/app-icon-no-background-full-white.png';
import { spaceHref } from '#lib/components/cards/spaceHref.ts';
import { spaceKind } from '#lib/domain/commentNotes.ts';
import type { Charter } from '#lib/domain/projects.ts';
import { tagRef } from '#lib/domain/scopes.ts';
import { t } from '#lib/i18n/index.ts';
import { scopeImageUrl } from '#lib/services/scopeImages.ts';

import { loadFonts, loadImage } from './canvasKit.ts';
import type { CardData } from './cardParts.ts';
import { qrMatrix } from './qr.ts';

/** The card's facts: the charter's title, first channel, type, summary and place, its page's
 * link as a QR code, and its picture, cover and the app's logo as loaded images. */
export async function cardData(id: string, charter: Charter): Promise<CardData> {
  const url = new URL(spaceHref(charter.mode, id), location.origin).href;
  const [picture, cover, logo] = await Promise.all([
    charter.picture ? scopeImageUrl(charter.picture).then(loadImage, () => null) : null,
    charter.cover ? scopeImageUrl(charter.cover).then(loadImage, () => null) : null,
    loadImage(brandLogo),
    loadFonts(),
  ]);
  const tag = charter.tags[0];
  const kind = t(spaceKind(charter.mode));
  return {
    title: charter.title,
    kind: tag ? `#${tagRef(tag).label} · ${kind}` : kind,
    summary: charter.description,
    // "Fitzroy Town Hall, 201, Napier Street, …" → its first two parts.
    place: charter.location_label.split(',').slice(0, 2).join(',').trim(),
    url,
    qr: qrMatrix(url),
    picture,
    cover,
    logo,
  };
}

/** Hands the file to the phone's share sheet (Save Image, AirDrop, apps) where it can take
 * files, else downloads it. */
export async function saveFile(blob: Blob, name: string): Promise<void> {
  const file = new File([blob], name, { type: blob.type });
  const phone = matchMedia('(pointer: coarse)').matches;
  if (phone && navigator.canShare?.({ files: [file] })) {
    try {
      await navigator.share({ files: [file] });
      return;
    } catch (e) {
      if (e instanceof DOMException && e.name === 'AbortError') return;
    }
  }
  const href = URL.createObjectURL(blob);
  const a = document.createElement('a');
  a.href = href;
  a.download = name;
  document.body.append(a);
  a.click();
  a.remove();
  setTimeout(() => URL.revokeObjectURL(href), 10_000);
}

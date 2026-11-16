import { AVATAR_MAX_BYTES, COVER_MAX_BYTES, type Avatar } from '#lib/domain/people.ts';

/** Centre-crop a picked image to `width`×`height` and shrink it to a WebP/JPEG under `max`. */
async function shrink(file: File, width: number, height: number, max: number): Promise<Avatar> {
  const bitmap = await createImageBitmap(file);
  const scale = Math.min(bitmap.width / width, bitmap.height / height);
  const [sw, sh] = [width * scale, height * scale];
  const canvas = document.createElement('canvas');
  canvas.width = width;
  canvas.height = height;
  const ctx = canvas.getContext('2d');
  if (!ctx) throw new Error('This browser cannot resize images.');
  ctx.drawImage(
    bitmap,
    (bitmap.width - sw) / 2,
    (bitmap.height - sh) / 2,
    sw,
    sh,
    0,
    0,
    width,
    height,
  );

  for (const [mime, quality] of [
    ['image/webp', 0.85],
    ['image/jpeg', 0.8],
    ['image/jpeg', 0.6],
  ] as const) {
    const blob = await new Promise<Blob | null>((resolve) => canvas.toBlob(resolve, mime, quality));
    if (blob && blob.type === mime && blob.size <= max) {
      return { mime, bytes: new Uint8Array(await blob.arrayBuffer()) };
    }
  }
  throw new Error('That image is too large even after shrinking. Try another one.');
}

/** A square profile photo, 256px. */
export const toAvatar = (file: File): Promise<Avatar> => shrink(file, 256, 256, AVATAR_MAX_BYTES);

/** A 3:1 cover photo, 1200×400 (ADR 0107). */
export const toCover = (file: File): Promise<Avatar> => shrink(file, 1200, 400, COVER_MAX_BYTES);

import { AVATAR_MAX_BYTES, type Avatar } from '#lib/domain/people.ts';

const SIDE = 256;

/** Square-crop and shrink a picked image to a small WebP/JPEG under the avatar limit. */
export async function toAvatar(file: File): Promise<Avatar> {
  const bitmap = await createImageBitmap(file);
  const side = Math.min(bitmap.width, bitmap.height);
  const canvas = document.createElement('canvas');
  canvas.width = SIDE;
  canvas.height = SIDE;
  const ctx = canvas.getContext('2d');
  if (!ctx) throw new Error('This browser cannot resize images.');
  const sx = (bitmap.width - side) / 2;
  const sy = (bitmap.height - side) / 2;
  ctx.drawImage(bitmap, sx, sy, side, side, 0, 0, SIDE, SIDE);

  for (const [mime, quality] of [
    ['image/webp', 0.85],
    ['image/jpeg', 0.8],
    ['image/jpeg', 0.6],
  ] as const) {
    const blob = await new Promise<Blob | null>((resolve) => canvas.toBlob(resolve, mime, quality));
    if (blob && blob.type === mime && blob.size <= AVATAR_MAX_BYTES) {
      return { mime, bytes: new Uint8Array(await blob.arrayBuffer()) };
    }
  }
  throw new Error('That image is too large even after shrinking. Try another one.');
}

import { callCell, conversationCell, stringToHash } from '#lib/holo/index.ts';

/** A group's managers and photo (ADR 0122): any manager makes another member a manager or
 * stops someone being one, and sets the photo (already cropped and shrunk). */
const chat = async <T>(seed: string, fn: string, payload?: unknown): Promise<T> =>
  callCell<T>(await conversationCell(seed), 'messages', fn, payload);

export async function setGroupManager(seed: string, agent: string, on: boolean): Promise<void> {
  await chat<null>(seed, 'set_manager', { agent: stringToHash(agent), on });
}

/** The photo as a small JPEG: at most 512 px, re-encoded until it fits the zome's 96 KB. */
export async function setGroupPicture(seed: string, photo: Blob): Promise<void> {
  await chat<null>(seed, 'set_picture', await shrink(photo));
}

async function shrink(photo: Blob): Promise<Uint8Array> {
  const image = await createImageBitmap(photo);
  const side = Math.min(512, Math.max(image.width, image.height));
  const scale = side / Math.max(image.width, image.height);
  const canvas = new OffscreenCanvas(
    Math.round(image.width * scale),
    Math.round(image.height * scale),
  );
  canvas.getContext('2d')?.drawImage(image, 0, 0, canvas.width, canvas.height);
  for (const quality of [0.85, 0.7, 0.55, 0.4]) {
    const blob = await canvas.convertToBlob({ type: 'image/jpeg', quality });
    if (blob.size <= 96 * 1024) return new Uint8Array(await blob.arrayBuffer());
  }
  throw new Error('That photo is too large. Try a smaller one.');
}

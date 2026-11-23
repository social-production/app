/**
 * Placing a chosen photo before it is saved (Owner Notes Mockups §3): a page asks with
 * `cropPhoto`, the shell's crop sheet answers with the placed image, or `null` when closed.
 */
export type CropShape = 'round' | 'wide';

export const photoCrop = $state<{ file: File | null; shape: CropShape }>({
  file: null,
  shape: 'round',
});

let done: ((file: File | null) => void) | null = null;

export function cropPhoto(file: File, shape: CropShape): Promise<File | null> {
  // Browser tests upload fixed files; they get them back unplaced.
  if (typeof navigator !== 'undefined' && navigator.webdriver) return Promise.resolve(file);
  done?.(null);
  photoCrop.shape = shape;
  photoCrop.file = file;
  return new Promise((resolve) => (done = resolve));
}

export function finishCrop(result: File | null): void {
  photoCrop.file = null;
  done?.(result);
  done = null;
}

/**
 * The crop screen's sums (Edit, Plan and Meetup mockups §A, owner 2026-11-24): an image shown
 * at `zoom` times the scale that just covers the frame, moved by `x`, `y` (CSS pixels from
 * centred), and the part of the image inside the frame.
 */

export interface CropView {
  zoom: number;
  x: number;
  y: number;
}

export const MAX_ZOOM = 4;

/** The scale at which the image just covers a `frameW` × `frameH` frame. */
export function coverScale(imgW: number, imgH: number, frameW: number, frameH: number): number {
  return Math.max(frameW / imgW, frameH / imgH);
}

/** Keeps the image covering the frame: no gap may show at any edge. */
export function clampView(
  view: CropView,
  img: { w: number; h: number },
  frame: { w: number; h: number },
): CropView {
  const zoom = Math.min(MAX_ZOOM, Math.max(1, view.zoom));
  const scale = coverScale(img.w, img.h, frame.w, frame.h) * zoom;
  const slackX = Math.max(0, (img.w * scale - frame.w) / 2);
  const slackY = Math.max(0, (img.h * scale - frame.h) / 2);
  return {
    zoom,
    x: Math.min(slackX, Math.max(-slackX, view.x)),
    y: Math.min(slackY, Math.max(-slackY, view.y)),
  };
}

/** The image's own pixels inside the frame: [sx, sy, sw, sh]. */
export function sourceRect(
  view: CropView,
  img: { w: number; h: number },
  frame: { w: number; h: number },
): [number, number, number, number] {
  const scale = coverScale(img.w, img.h, frame.w, frame.h) * view.zoom;
  const sw = frame.w / scale;
  const sh = frame.h / scale;
  const sx = img.w / 2 - view.x / scale - sw / 2;
  const sy = img.h / 2 - view.y / scale - sh / 2;
  return [sx, sy, sw, sh];
}

/** The framed part as a JPEG file `outW` wide, ready for `ScopeArt.stage`. */
export async function cropToFile(
  img: HTMLImageElement,
  view: CropView,
  frame: { w: number; h: number },
  outW: number,
  name: string,
): Promise<File> {
  const outH = Math.round((outW * frame.h) / frame.w);
  const canvas = document.createElement('canvas');
  canvas.width = outW;
  canvas.height = outH;
  const ctx = canvas.getContext('2d');
  if (!ctx) throw new Error('This browser cannot crop images.');
  const size = { w: img.naturalWidth, h: img.naturalHeight };
  ctx.drawImage(img, ...sourceRect(view, size, frame), 0, 0, outW, outH);
  const blob = await new Promise<Blob>((resolve, reject) =>
    canvas.toBlob(
      (b) => (b ? resolve(b) : reject(new Error('Could not crop the image.'))),
      'image/jpeg',
      0.92,
    ),
  );
  return new File([blob], name, { type: 'image/jpeg' });
}

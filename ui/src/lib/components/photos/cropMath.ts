/** The geometry behind the photo crop sheet: an image covering a frame, panned and zoomed. */
export interface Fit {
  /** The scale at which the image just covers the frame (zoom 1). */
  base: number;
  frameW: number;
  frameH: number;
  imgW: number;
  imgH: number;
}

/** Keeps the image covering the frame: offsets are the image's top-left inside the frame. */
export function clampOffset(fit: Fit, zoom: number, x: number, y: number): [number, number] {
  const w = fit.imgW * fit.base * zoom;
  const h = fit.imgH * fit.base * zoom;
  return [Math.min(0, Math.max(fit.frameW - w, x)), Math.min(0, Math.max(fit.frameH - h, y))];
}

/** The part of the image the frame shows, in the image's own pixels. */
export function sourceRect(fit: Fit, zoom: number, x: number, y: number) {
  const scale = fit.base * zoom;
  return { sx: -x / scale, sy: -y / scale, sw: fit.frameW / scale, sh: fit.frameH / scale };
}

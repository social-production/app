<script lang="ts">
  import { t } from '#lib/i18n/index.ts';
  import OverlaySheet from '#lib/components/overlay/OverlaySheet.svelte';
  import '#lib/components/styles/ux/photocrop.css';

  import { clampOffset, sourceRect, type Fit } from './cropMath.ts';

  /** Drag and zoom a chosen photo inside its frame, then use it (Owner Notes Mockups §3): a
   * round frame for a person's photo, a wide one for a cover. */
  interface Props {
    file: File | null;
    shape: 'round' | 'wide';
    onDone: (file: File | null) => void;
  }

  let { file, shape, onDone }: Props = $props();

  const frameW = $derived(shape === 'round' ? 280 : 330);
  const frameH = $derived(shape === 'round' ? 280 : 120);
  let image = $state<HTMLImageElement | null>(null);
  let url = $state<string | null>(null);
  let zoom = $state(1);
  let x = $state(0);
  let y = $state(0);

  const fit = $derived<Fit | null>(
    image
      ? {
          base: Math.max(frameW / image.naturalWidth, frameH / image.naturalHeight),
          frameW,
          frameH,
          imgW: image.naturalWidth,
          imgH: image.naturalHeight,
        }
      : null,
  );

  $effect(() => {
    if (!file) return;
    const next = URL.createObjectURL(file);
    url = next;
    const img = new Image();
    img.onload = () => {
      image = img;
      zoom = 1;
      const base = Math.max(frameW / img.naturalWidth, frameH / img.naturalHeight);
      x = (frameW - img.naturalWidth * base) / 2;
      y = (frameH - img.naturalHeight * base) / 2;
    };
    img.src = next;
    return () => {
      URL.revokeObjectURL(next);
      image = null;
    };
  });

  function place(nx: number, ny: number, nz = zoom): void {
    if (!fit) return;
    // Zoom about the frame's centre, so the middle stays put.
    const cx = frameW / 2;
    const cy = frameH / 2;
    const ratio = nz / zoom;
    const zx = cx - (cx - nx) * ratio;
    const zy = cy - (cy - ny) * ratio;
    zoom = nz;
    [x, y] = clampOffset(fit, nz, zx, zy);
  }

  // Plain on purpose: touch points are bookkeeping, nothing redraws from them.
  const pointers: Record<number, { x: number; y: number } | undefined> = {};
  let pinch = 0;

  function down(event: PointerEvent): void {
    (event.currentTarget as HTMLElement).setPointerCapture(event.pointerId);
    pointers[event.pointerId] = { x: event.clientX, y: event.clientY };
    pinch = spread();
  }

  function spread(): number {
    const [a, b] = Object.values(pointers).filter((p) => !!p);
    return a && b ? Math.hypot(a.x - b.x, a.y - b.y) : 0;
  }

  function move(event: PointerEvent): void {
    const last = pointers[event.pointerId];
    if (!last) return;
    pointers[event.pointerId] = { x: event.clientX, y: event.clientY };
    if (Object.values(pointers).filter((p) => !!p).length === 2) {
      const now = spread();
      if (pinch) place(x, y, Math.min(4, Math.max(1, (zoom * now) / pinch)));
      pinch = now;
      return;
    }
    place(x + event.clientX - last.x, y + event.clientY - last.y);
  }

  function up(event: PointerEvent): void {
    pointers[event.pointerId] = undefined;
    pinch = spread();
  }

  async function use(): Promise<void> {
    if (!image || !fit || !file) return;
    const out = Math.min(2, Math.max(1, image.naturalWidth / (frameW * zoom)));
    const canvas = document.createElement('canvas');
    canvas.width = Math.round(frameW * out);
    canvas.height = Math.round(frameH * out);
    const r = sourceRect(fit, zoom, x, y);
    canvas
      .getContext('2d')
      ?.drawImage(image, r.sx, r.sy, r.sw, r.sh, 0, 0, canvas.width, canvas.height);
    const blob = await new Promise<Blob | null>((ok) => canvas.toBlob(ok, 'image/jpeg', 0.9));
    onDone(
      blob
        ? new File([blob], file.name.replace(/\.\w+$/, '') + '.jpg', { type: 'image/jpeg' })
        : file,
    );
  }
</script>

<OverlaySheet
  open={!!file}
  title={t('Place your photo')}
  labelledById="photo-crop-title"
  rootClass="photo-crop-sheet"
  onClose={() => onDone(null)}
>
  <div class="crop-stage">
    <div
      class="crop-frame"
      class:round={shape === 'round'}
      style={`width: ${frameW}px; height: ${frameH}px;`}
      role="application"
      aria-label={t('Drag to move, pinch or use the slider to zoom')}
      onpointerdown={down}
      onpointermove={move}
      onpointerup={up}
      onpointercancel={up}
      onwheel={(e) => {
        e.preventDefault();
        place(x, y, Math.min(4, Math.max(1, zoom - e.deltaY / 400)));
      }}
    >
      {#if url && fit}
        <img
          alt=""
          src={url}
          draggable="false"
          style={`width: ${fit.imgW * fit.base * zoom}px; height: ${fit.imgH * fit.base * zoom}px; transform: translate(${x}px, ${y}px);`}
        />
      {/if}
    </div>
    <p class="crop-hint">{t('Drag to move · pinch to zoom')}</p>
    <input
      class="crop-zoom"
      type="range"
      min="1"
      max="4"
      step="0.01"
      value={zoom}
      aria-label={t('Zoom')}
      oninput={(e) => place(x, y, Number(e.currentTarget.value))}
    />
  </div>
  {#snippet footer()}
    <div class="sheet-actions">
      <button class="sheet-submit" type="button" disabled={!image} onclick={() => void use()}
        >{t('Use photo')}</button
      >
    </div>
  {/snippet}
</OverlaySheet>

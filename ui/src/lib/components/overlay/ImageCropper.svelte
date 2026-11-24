<script lang="ts">
  import { t } from '#lib/i18n/index.ts';
  import { portal } from '#lib/components/actions/portal.ts';

  import { clampView, coverScale, cropToFile, MAX_ZOOM, type CropView } from './cropMath.ts';

  import '#lib/components/styles/ux/cropper.css';

  /**
   * Move and zoom a cover or picture before it is used (Edit, Plan and Meetup mockups §A, owner
   * 2026-11-24: "crop it how you want"). Drag to move, pinch or the slider to zoom; Done hands
   * back the framed part as a new file.
   */
  interface Props {
    /** An object URL or image URL to crop. */
    src: string;
    slot: 'picture' | 'cover';
    onDone: (file: File) => void;
    onCancel: () => void;
    /** Pick a different file instead. */
    onReplace?: (() => void) | undefined;
  }

  let { src, slot, onDone, onCancel, onReplace }: Props = $props();

  let img = $state<HTMLImageElement>();
  let frameEl = $state<HTMLDivElement>();
  let natural = $state<{ w: number; h: number } | null>(null);
  let frame = $state({ w: 1, h: 1 });
  let view = $state<CropView>({ zoom: 1, x: 0, y: 0 });
  let busy = $state(false);
  // Touch points while dragging; nothing renders from them.
  // eslint-disable-next-line svelte/prefer-svelte-reactivity -- not state
  const pointers = new Map<number, { x: number; y: number }>();
  let pinch: { distance: number; zoom: number } | null = null;

  const scale = $derived(
    natural ? coverScale(natural.w, natural.h, frame.w, frame.h) * view.zoom : 1,
  );

  $effect(() => {
    if (!frameEl) return;
    const observer = new ResizeObserver(() => {
      if (frameEl) frame = { w: frameEl.clientWidth, h: frameEl.clientHeight };
    });
    observer.observe(frameEl);
    return () => observer.disconnect();
  });

  function set(next: CropView): void {
    if (natural) view = clampView(next, natural, frame);
  }

  function down(event: PointerEvent): void {
    frameEl?.setPointerCapture(event.pointerId);
    pointers.set(event.pointerId, { x: event.clientX, y: event.clientY });
    if (pointers.size === 2) pinch = { distance: spread(), zoom: view.zoom };
  }

  function move(event: PointerEvent): void {
    const last = pointers.get(event.pointerId);
    if (!last) return;
    pointers.set(event.pointerId, { x: event.clientX, y: event.clientY });
    if (pinch && pointers.size === 2)
      set({ ...view, zoom: (pinch.zoom * spread()) / pinch.distance });
    else if (pointers.size === 1)
      set({ ...view, x: view.x + event.clientX - last.x, y: view.y + event.clientY - last.y });
  }

  function up(event: PointerEvent): void {
    pointers.delete(event.pointerId);
    if (pointers.size < 2) pinch = null;
  }

  function spread(): number {
    const [a, b] = [...pointers.values()];
    return a && b ? Math.hypot(a.x - b.x, a.y - b.y) || 1 : 1;
  }

  function wheel(event: WheelEvent): void {
    event.preventDefault();
    set({ ...view, zoom: view.zoom * (event.deltaY < 0 ? 1.08 : 1 / 1.08) });
  }

  async function done(): Promise<void> {
    if (!img || !natural || busy) return;
    busy = true;
    try {
      const file = await cropToFile(img, view, frame, slot === 'cover' ? 1500 : 800, `${slot}.jpg`);
      onDone(file);
    } finally {
      busy = false;
    }
  }
</script>

<div class="cropper" use:portal role="dialog" aria-modal="true" aria-label={t('Adjust image')}>
  <header class="cropper-top">
    <button type="button" class="cropper-close" aria-label={t('Cancel')} onclick={onCancel}
      >×</button
    >
    <strong>{slot === 'cover' ? t('Adjust cover') : t('Adjust picture')}</strong>
    <span></span>
  </header>
  <div
    class="cropper-stage"
    bind:this={frameEl}
    class:round={slot === 'picture'}
    class:wide={slot === 'cover'}
    role="presentation"
    onpointerdown={down}
    onpointermove={move}
    onpointerup={up}
    onpointercancel={up}
    onwheel={wheel}
  >
    <img
      bind:this={img}
      {src}
      alt=""
      draggable="false"
      style:width={natural ? `${natural.w * scale}px` : undefined}
      style:transform={`translate(-50%, -50%) translate(${view.x}px, ${view.y}px)`}
      onload={() => {
        if (img) natural = { w: img.naturalWidth, h: img.naturalHeight };
        set({ zoom: 1, x: 0, y: 0 });
      }}
    />
    <span class="cropper-grid" aria-hidden="true"></span>
  </div>
  <p class="cropper-hint">{t('Drag to move · pinch to zoom')}</p>
  <label class="cropper-zoom">
    <span aria-hidden="true">−</span>
    <input
      type="range"
      min="1"
      max={MAX_ZOOM}
      step="0.01"
      value={view.zoom}
      aria-label={t('Zoom')}
      oninput={(e) => set({ ...view, zoom: Number(e.currentTarget.value) })}
    />
    <span aria-hidden="true">+</span>
  </label>
  <footer class="cropper-actions">
    {#if onReplace}
      <button type="button" class="cropper-replace" onclick={onReplace}
        >{t('Choose another')}</button
      >
    {:else}<span></span>{/if}
    <button type="button" class="cropper-done" disabled={!natural || busy} onclick={done}
      >{busy ? t('Working...') : t('Done')}</button
    >
  </footer>
</div>

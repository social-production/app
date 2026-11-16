<script lang="ts">
  import type { Snippet } from 'svelte';

  import { portal } from '#lib/components/actions/portal.ts';

  import { ZoomPan } from './zoomPan.svelte.ts';

  import '#lib/components/styles/photo-viewer.css';

  /** web's components/shared/PhotoViewer: a full-screen photo, wheel or pinch to zoom, drag
   * to pan, Escape or × to close. With several photos (a post's, Profiles and Comments
   * mockups §5) it counts them and swipes or arrows between them; double-tap zooms, a swipe
   * down closes, and `footer` sits along the bottom. */
  interface Props {
    url: string | null;
    alt?: string;
    onClose: () => void;
    /** This photo's place among `count` (from 0). */
    index?: number;
    count?: number;
    onStep?: ((delta: number) => void) | undefined;
    footer?: Snippet | undefined;
  }

  let { url, alt = 'Photo', onClose, index = 0, count = 1, onStep, footer }: Props = $props();
  let swipe: { x: number; y: number } | null = null;

  function step(delta: number): void {
    const next = index + delta;
    if (!onStep || next < 0 || next >= count) return;
    view.reset();
    onStep(delta);
  }

  function touchStart(event: TouchEvent): void {
    view.touchStart(event);
    const [only] = event.touches;
    swipe =
      event.touches.length === 1 && only && view.scale <= 1
        ? { x: only.clientX, y: only.clientY }
        : null;
  }

  function touchEnd(event: TouchEvent): void {
    view.touchEnd(event);
    const end = event.changedTouches[0];
    if (!swipe || !end || view.scale > 1) return;
    const [dx, dy] = [end.clientX - swipe.x, end.clientY - swipe.y];
    swipe = null;
    if (Math.abs(dx) > 50 && Math.abs(dx) > Math.abs(dy)) step(dx < 0 ? 1 : -1);
    else if (dy > 90) close();
  }

  function zoomToggle(): void {
    if (view.scale > 1) view.reset();
    else view.scale = 2;
  }

  const view = new ZoomPan();
  let image = $state<HTMLImageElement>();

  $effect(() => {
    view.image = image ?? null;
  });

  function close(): void {
    view.reset();
    onClose();
  }

  function keydown(event: KeyboardEvent): void {
    if (url && event.key === 'Escape') {
      event.preventDefault();
      close();
    }
    if (url && event.key === 'ArrowRight') step(1);
    if (url && event.key === 'ArrowLeft') step(-1);
  }

  // Svelte 5 attaches wheel and touchmove passively; zooming needs preventDefault.
  function gestures(node: HTMLElement): { destroy: () => void } {
    const wheel = (e: WheelEvent): void => view.wheel(e);
    const move = (e: TouchEvent): void => view.touchMove(e);
    node.addEventListener('wheel', wheel, { passive: false });
    node.addEventListener('touchmove', move, { passive: false });
    return {
      destroy: () => {
        node.removeEventListener('wheel', wheel);
        node.removeEventListener('touchmove', move);
      },
    };
  }
</script>

<svelte:window onkeydown={keydown} />

{#if url}
  <div class="photo-viewer" role="presentation" use:portal>
    <button class="scrim" type="button" aria-label="Close photo" title="Close photo" onclick={close}
    ></button>
    <div
      aria-label={alt}
      aria-modal="true"
      class="frame"
      role="dialog"
      tabindex="-1"
      use:gestures
      onpointerdown={(e) => view.pointerDown(e)}
      onpointermove={(e) => view.pointerMove(e)}
      onpointerup={() => view.pointerUp()}
      onpointercancel={() => view.pointerUp()}
      ontouchstart={touchStart}
      ontouchend={touchEnd}
    >
      <img
        bind:this={image}
        {alt}
        src={url}
        style={`transform: translate(${view.x}px, ${view.y}px) scale(${view.scale})`}
        ondblclick={zoomToggle}
      />
      <button aria-label="Close photo" class="close" type="button" onclick={close}>×</button>
      {#if count > 1}
        <span class="photo-count">{index + 1} / {count}</span>
        {#if index > 0}
          <button
            class="photo-step prev"
            type="button"
            aria-label="Previous photo"
            onclick={() => step(-1)}>‹</button
          >
        {/if}
        {#if index < count - 1}
          <button
            class="photo-step next"
            type="button"
            aria-label="Next photo"
            onclick={() => step(1)}>›</button
          >
        {/if}
        <span class="photo-dots" aria-hidden="true">
          {#each Array.from({ length: count }, (_v, i) => i) as i (i)}<i class:on={i === index}
            ></i>{/each}
        </span>
      {/if}
      {#if footer}<div class="photo-footer">{@render footer()}</div>{/if}
    </div>
  </div>
{/if}

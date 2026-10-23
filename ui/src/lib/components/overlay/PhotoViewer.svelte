<script lang="ts">
  import { portal } from '#lib/components/actions/portal.ts';

  import { ZoomPan } from './zoomPan.svelte.ts';

  import '#lib/components/styles/photo-viewer.css';

  /** web's components/shared/PhotoViewer: a full-screen photo, wheel or pinch to zoom, drag
   * to pan, Escape or × to close. */
  interface Props {
    url: string | null;
    alt?: string;
    onClose: () => void;
  }

  let { url, alt = 'Photo', onClose }: Props = $props();

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
      ontouchstart={(e) => view.touchStart(e)}
      ontouchend={(e) => view.touchEnd(e)}
    >
      <img
        bind:this={image}
        {alt}
        src={url}
        style={`transform: translate(${view.x}px, ${view.y}px) scale(${view.scale})`}
      />
      <button aria-label="Close photo" class="close" type="button" onclick={close}>×</button>
    </div>
  </div>
{/if}

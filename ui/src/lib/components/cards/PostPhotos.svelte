<script lang="ts">
  import PhotoViewer from '#lib/components/overlay/PhotoViewer.svelte';
  import type { ChatAttachment } from '#lib/domain/chat.ts';

  import { postPhotoSource } from './postPhotoSource.ts';

  import '#lib/components/styles/ux/photos.css';

  /**
   * A post's photos (ADR 0101): one to four, in a rounded grid under its text. They load
   * when the post scrolls into view. On the post's page a photo opens full screen; in the
   * feed the row's own link opens the post.
   */
  let { post, open = false }: { post: string; open?: boolean } = $props();

  const source = postPhotoSource();
  let photos = $state<ChatAttachment[]>([]);
  let viewing = $state<{ url: string; name: string } | null>(null);

  function load(node: HTMLElement): { destroy: () => void } {
    const read = (): void => {
      void source.list(post).then(
        (list) => (photos = list),
        () => undefined,
      );
    };
    if (typeof IntersectionObserver === 'undefined') {
      read();
      return { destroy: () => undefined };
    }
    const seen = new IntersectionObserver(
      (entries) => {
        if (entries.some((e) => e.isIntersecting)) {
          seen.disconnect();
          read();
        }
      },
      { rootMargin: '300px' },
    );
    seen.observe(node);
    return { destroy: () => seen.disconnect() };
  }

  function src(node: HTMLImageElement, photo: ChatAttachment): void {
    void source.url(post, photo).then(
      (url) => (node.src = url),
      () => undefined,
    );
  }

  async function view(photo: ChatAttachment): Promise<void> {
    const url = await source.url(post, photo).catch(() => null);
    if (url) viewing = { url, name: photo.name };
  }
</script>

<div class="post-photos-anchor" use:load>
  {#if photos.length > 0}
    <div class="post-photos" class:open data-count={photos.length}>
      {#each photos as photo, i (i)}
        {#if open}
          <button type="button" aria-label={`View photo ${i + 1}`} onclick={() => void view(photo)}>
            <img alt={`Photo ${i + 1} of ${photos.length}`} use:src={photo} />
          </button>
        {:else}
          <img alt={`Photo ${i + 1} of ${photos.length}`} use:src={photo} />
        {/if}
      {/each}
    </div>
  {/if}
</div>
{#if viewing}
  <PhotoViewer url={viewing.url} alt={viewing.name} onClose={() => (viewing = null)} />
{/if}

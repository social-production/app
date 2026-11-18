<script lang="ts">
  import { plural } from '#lib/i18n/index.ts';
  import PhotoViewer from '#lib/components/overlay/PhotoViewer.svelte';
  import type { ChatAttachment } from '#lib/domain/chat.ts';

  import type { ResolvedPathname } from '$app/types';

  import { postPhotoSource } from './postPhotoSource.ts';

  import '#lib/components/styles/ux/photos.css';

  /**
   * A post's photos (ADR 0101): one to four, in a rounded grid under its text. They load
   * when the post scrolls into view. A photo opens full screen, where you swipe between them
   * (Profiles and Comments mockups §5); in the feed (`postHref`) a bar along the bottom shows
   * the text and opens the post.
   */
  interface Props {
    post: string;
    /** On the post's own page (no footer). */
    open?: boolean;
    postHref?: ResolvedPathname | null;
    caption?: string;
    commentCount?: number;
  }

  let { post, open = false, postHref = null, caption = '', commentCount = 0 }: Props = $props();

  const source = postPhotoSource();
  let photos = $state<ChatAttachment[]>([]);
  let viewing = $state<{ url: string; name: string; index: number } | null>(null);

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

  async function view(index: number): Promise<void> {
    const photo = photos[index];
    if (!photo) return;
    const url = await source.url(post, photo).catch(() => null);
    if (url) viewing = { url, name: photo.name, index };
  }
</script>

<div class="post-photos-anchor" use:load>
  {#if photos.length > 0}
    <div class="post-photos" class:open data-count={photos.length}>
      {#each photos as photo, i (i)}
        <button type="button" aria-label={`View photo ${i + 1}`} onclick={() => void view(i)}>
          <img alt={`Photo ${i + 1} of ${photos.length}`} use:src={photo} />
        </button>
      {/each}
    </div>
  {/if}
</div>
{#if viewing}
  <PhotoViewer
    url={viewing.url}
    alt={viewing.name}
    index={viewing.index}
    count={photos.length}
    onStep={(delta) => void view((viewing?.index ?? 0) + delta)}
    onClose={() => (viewing = null)}
    footer={postHref ? postFooter : undefined}
  />
{/if}

{#snippet postFooter()}
  <p class="photo-caption">{caption}</p>
  <!-- eslint-disable-next-line svelte/no-navigation-without-resolve -- postHref arrives resolved -->
  <a class="photo-open" href={postHref}
    >Open post{commentCount ? ` · ${plural(commentCount, '{n} comment', '{n} comments')}` : ''} ›</a
  >
{/snippet}

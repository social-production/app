<script lang="ts">
  import { shareTargets } from '#lib/domain/share.ts';

  /**
   * web's ShareUserMenu, the "share outside the app" half: the device's share sheet where there
   * is one, Copy link, and links to the usual places otherwise (styles in share.css).
   */
  let { url, title }: { url: string; title: string } = $props();

  const nativeShare = typeof navigator !== 'undefined' && typeof navigator.share === 'function';
  let copyLabel = $state('Copy link');
  let timer: ReturnType<typeof setTimeout> | null = null;

  /** web's shareLink: the share sheet, else the link copied. */
  async function shareVia(): Promise<void> {
    try {
      await navigator.share({ url, title });
    } catch (e) {
      if (e instanceof DOMException && e.name === 'AbortError') return;
      await copy();
    }
  }

  async function copy(): Promise<void> {
    try {
      await navigator.clipboard.writeText(url);
      copyLabel = 'Copied';
    } catch {
      copyLabel = 'Copy failed';
    }
    if (timer) clearTimeout(timer);
    timer = setTimeout(() => {
      copyLabel = 'Copy link';
      timer = null;
    }, 1600);
  }
</script>

<section class="share-out" aria-label="Share outside the app">
  {#if title}<p class="share-subject">{title}</p>{/if}
  <div class="out-actions">
    {#if nativeShare}
      <button class="out-button primary" type="button" onclick={() => void shareVia()}>
        <svg aria-hidden="true" viewBox="0 0 24 24" width="16" height="16"
          ><path
            d="M12 3v12M7 8l5-5 5 5M5 13v6a2 2 0 0 0 2 2h10a2 2 0 0 0 2-2v-6"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
            stroke-linecap="round"
            stroke-linejoin="round"
          /></svg
        >
        Share via…
      </button>
    {/if}
    <button
      class="out-button"
      class:copied={copyLabel === 'Copied'}
      type="button"
      title={url}
      onclick={() => void copy()}
    >
      <svg aria-hidden="true" viewBox="0 0 24 24" width="16" height="16"
        ><path
          d="M10 14a4 4 0 0 0 5.7 0l3-3a4 4 0 0 0-5.7-5.7l-1 1M14 10a4 4 0 0 0-5.7 0l-3 3a4 4 0 0 0 5.7 5.7l1-1"
          fill="none"
          stroke="currentColor"
          stroke-width="2"
          stroke-linecap="round"
        /></svg
      >
      {copyLabel}
    </button>
  </div>
  {#if !nativeShare}
    <!-- eslint-disable svelte/no-navigation-without-resolve -- links off the app -->
    <div class="share-out-row">
      {#each shareTargets({ url, title }) as target (target.id)}
        <a
          class="share-tile"
          href={target.href}
          target="_blank"
          rel="noopener noreferrer"
          title={`Share on ${target.label}`}
          aria-label={`Share on ${target.label}`}
        >
          <span class={`tile-icon ${target.id}`} aria-hidden="true">
            {#if target.id === 'whatsapp'}
              <svg viewBox="0 0 24 24" width="20" height="20"
                ><path
                  d="M4 20l1.3-4A8 8 0 1 1 8 18.7L4 20z"
                  fill="none"
                  stroke="currentColor"
                  stroke-width="2"
                  stroke-linejoin="round"
                /><path
                  d="M9 9.5c.3 2 2 3.8 4.2 4.4l1-1.1 1.8.8c-.2 1-1 1.6-2 1.5A6 6 0 0 1 8.4 10c-.1-1 .5-1.8 1.5-2l.8 1.8-1 .9"
                  fill="currentColor"
                /></svg
              >
            {:else if target.id === 'telegram'}
              <svg viewBox="0 0 24 24" width="20" height="20"
                ><path
                  d="M21 4L3 11l6 2 2 6 3-4 5 4 2-15zM9 13l9-6-7 8"
                  fill="none"
                  stroke="currentColor"
                  stroke-width="2"
                  stroke-linejoin="round"
                /></svg
              >
            {:else if target.id === 'email'}
              <svg viewBox="0 0 24 24" width="20" height="20"
                ><rect
                  x="3"
                  y="5"
                  width="18"
                  height="14"
                  rx="2"
                  fill="none"
                  stroke="currentColor"
                  stroke-width="2"
                /><path d="M3 7l9 6 9-6" fill="none" stroke="currentColor" stroke-width="2" /></svg
              >
            {:else if target.id === 'x'}
              <svg viewBox="0 0 24 24" width="18" height="18"
                ><path
                  d="M5 4l14 16M19 4L5 20"
                  stroke="currentColor"
                  stroke-width="2.4"
                  stroke-linecap="round"
                /></svg
              >
            {:else}
              <svg viewBox="0 0 24 24" width="20" height="20"
                ><path
                  d="M14 8h3V4h-3a4 4 0 0 0-4 4v3H7v4h3v6h4v-6h3l1-4h-4V8z"
                  fill="currentColor"
                /></svg
              >
            {/if}
          </span>
        </a>
      {/each}
    </div>
    <!-- eslint-enable svelte/no-navigation-without-resolve -->
  {/if}
</section>

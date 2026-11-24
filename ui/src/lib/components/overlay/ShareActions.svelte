<script lang="ts">
  import { t } from '#lib/i18n/index.ts';

  /**
   * The share sheet's three ways out (Owner Notes Mockups §6): Copy link, Share via… (the
   * phone's own sheet, or the link copied where there is none) and Quote in a post.
   */
  interface Props {
    url: string;
    title: string;
    onQuote?: (() => void) | null;
  }

  let { url, title, onQuote = null }: Props = $props();

  const nativeShare = typeof navigator !== 'undefined' && typeof navigator.share === 'function';
  let copied = $state<'' | 'Copied' | 'Copy failed'>('');
  let timer: ReturnType<typeof setTimeout> | null = null;

  async function copy(): Promise<void> {
    try {
      await navigator.clipboard.writeText(url);
      copied = 'Copied';
    } catch {
      copied = 'Copy failed';
    }
    if (timer) clearTimeout(timer);
    timer = setTimeout(() => (copied = ''), 1600);
  }

  async function via(): Promise<void> {
    try {
      await navigator.share({ url, title });
    } catch (e) {
      if (e instanceof DOMException && e.name === 'AbortError') return;
      await copy();
    }
  }
</script>

<div class="share-actions">
  <button class="share-action" class:done={copied === 'Copied'} type="button" onclick={copy}>
    <span class="share-action-icon" aria-hidden="true">
      <svg viewBox="0 0 24 24" width="20" height="20"
        ><path
          d="M10 14a4 4 0 0 0 5.7 0l3-3a4 4 0 0 0-5.7-5.7l-1 1M14 10a4 4 0 0 0-5.7 0l-3 3a4 4 0 0 0 5.7 5.7l1-1"
          fill="none"
          stroke="currentColor"
          stroke-width="2"
          stroke-linecap="round"
        /></svg
      >
    </span>
    <span role="status">{t(copied || 'Copy link')}</span>
  </button>
  {#if nativeShare}
    <button class="share-action" type="button" onclick={via}>
      <span class="share-action-icon" aria-hidden="true">
        <svg viewBox="0 0 24 24" width="20" height="20"
          ><path
            d="M12 15V4.5M8 8l4-3.5L16 8M6 12.5V18a1 1 0 0 0 1 1h10a1 1 0 0 0 1-1v-5.5"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
            stroke-linecap="round"
            stroke-linejoin="round"
          /></svg
        >
      </span>
      <span>{t('Share via…')}</span>
    </button>
  {/if}
  {#if onQuote}
    <button class="share-action" type="button" onclick={onQuote}>
      <span class="share-action-icon" aria-hidden="true">
        <svg viewBox="0 0 24 24" width="20" height="20"
          ><path
            d="M5 5h14v10H9l-4 4V5z"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
            stroke-linejoin="round"
          /><path
            d="M9 9h6M9 12h4"
            stroke="currentColor"
            stroke-width="2"
            stroke-linecap="round"
          /></svg
        >
      </span>
      <span>{t('Quote in a post')}</span>
    </button>
  {/if}
</div>

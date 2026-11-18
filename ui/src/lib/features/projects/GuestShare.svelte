<script lang="ts">
  import { t } from '#lib/i18n/index.ts';
  /** Share for someone browsing signed out: the device's share sheet, or the link copied. */
  let { title }: { title: string } = $props();

  let copied = $state(false);

  async function share(): Promise<void> {
    const url = `${location.origin}${location.pathname}`;
    if (typeof navigator.share === 'function') {
      await navigator.share({ url, title }).catch(() => {});
      return;
    }
    await navigator.clipboard?.writeText(url).catch(() => {});
    copied = true;
    setTimeout(() => (copied = false), 1500);
  }
</script>

<button
  type="button"
  class="share-button"
  aria-label={copied ? t('Link copied') : t('Share')}
  title={copied ? t('Link copied') : t('Share')}
  onclick={() => void share()}
>
  <svg aria-hidden="true" viewBox="0 0 24 24" width="18" height="18"
    ><path
      d="M12 3v12M7 8l5-5 5 5M5 13v6a2 2 0 0 0 2 2h10a2 2 0 0 0 2-2v-6"
      fill="none"
      stroke="currentColor"
      stroke-width="1.8"
      stroke-linecap="round"
      stroke-linejoin="round"
    /></svg
  >
</button>

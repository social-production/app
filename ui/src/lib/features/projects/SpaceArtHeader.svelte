<script lang="ts">
  import { t } from '#lib/i18n/index.ts';

  import '#lib/components/styles/ux/spaceart.css';

  /**
   * A project's, service's or event's cover at the top of its page (ADR 0117), edge to edge
   * like a profile (Details Header Mockups, B). Its picture sits beside the channel line
   * below, in the header's tile (FeedCardTop's `picture`). Tapping the cover opens it full
   * screen (owner, 2026-11-24).
   */
  let {
    cover,
    url,
    onOpen,
  }: { cover: string | null | undefined; url: string | null; onOpen: () => void } = $props();
</script>

{#if cover}
  <button
    class="space-art-cover"
    type="button"
    aria-label={t('View cover')}
    disabled={!url}
    style={url ? `background-image: url(${url})` : undefined}
    onclick={onOpen}
  ></button>
{/if}

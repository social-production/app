<script lang="ts">
  import { ScopeArt } from './scopes/scopeArt.svelte.ts';

  import '#lib/components/styles/ux/spaceart.css';

  /**
   * A project's, service's or event's cover at the top of its page (ADR 0117), edge to edge
   * like a profile (Details Header Mockups, B). Its picture sits beside the channel line
   * below, in the header's tile (FeedCardTop's `picture`).
   */
  let { cover }: { cover: string | null | undefined } = $props();

  const art = new ScopeArt();
  $effect(() => {
    void art.show(null, cover);
  });
</script>

{#if cover}
  <div
    class="space-art-cover"
    style={art.coverUrl ? `background-image: url(${art.coverUrl})` : undefined}
  ></div>
{/if}

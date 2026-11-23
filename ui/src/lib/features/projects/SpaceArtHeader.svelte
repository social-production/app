<script lang="ts">
  import { ScopeArt } from './scopes/scopeArt.svelte.ts';

  import '#lib/components/styles/ux/spaceart.css';

  /**
   * A project's, service's or event's cover with its picture over the corner (ADR 0117), at
   * the top of its page. Nothing shows until it has one or the other.
   */
  let { picture, cover }: { picture: string | null | undefined; cover: string | null | undefined } =
    $props();

  const art = new ScopeArt();
  $effect(() => {
    void art.show(picture, cover);
  });
</script>

{#if art.coverUrl || art.pictureUrl}
  <div class="space-art" class:no-cover={!art.coverUrl}>
    {#if art.coverUrl}
      <div class="space-art-cover" style={`background-image: url(${art.coverUrl})`}></div>
    {/if}
    {#if art.pictureUrl}<img class="space-art-picture" src={art.pictureUrl} alt="" />{/if}
  </div>
{/if}

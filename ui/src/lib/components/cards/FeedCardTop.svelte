<script lang="ts">
  import { MediaQuery } from 'svelte/reactivity';

  import { resolve } from '$app/paths';

  /**
   * web's FeedCardTop with TagList (plain): tags left (3 on desktop, 1 on phones, "+N" for
   * the rest), the stage right. Tags link to search until channels arrive (M5).
   */
  let { tags, stage }: { tags: string[]; stage: string } = $props();

  const phone = new MediaQuery('(max-width: 760px)');
  let expanded = $state(false);
  const visible = $derived(expanded ? tags : tags.slice(0, phone.current ? 1 : 3));
  const hidden = $derived(tags.length - visible.length);

  function showAll(event: MouseEvent): void {
    event.preventDefault();
    event.stopPropagation();
    expanded = true;
  }

  import '#lib/components/styles/cards.css';
</script>

{#if tags.length > 0 || stage}
  <div class="feed-card-top">
    <div class="feed-card-tags">
      {#if tags.length > 0}
        <div class="tag-list plain">
          {#each visible as tag, index (tag)}
            {#if index > 0}<span class="tag-sep" aria-hidden="true">,</span>{/if}
            <a class="tag-plain" href={resolve(`/search?q=${encodeURIComponent(tag)}`)}
              ><span>{tag}</span></a
            >
          {/each}
          {#if hidden > 0}
            <span class="tag-sep" aria-hidden="true">,</span>
            <button
              type="button"
              class="tag-overflow-btn"
              aria-label={`Show ${hidden} more tags`}
              onclick={showAll}>+{hidden}</button
            >
          {/if}
        </div>
      {/if}
    </div>
    <div class="feed-card-end">
      {#if stage}<span class="feed-card-stage">{stage}</span>{/if}
    </div>
  </div>
{/if}

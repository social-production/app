<script lang="ts">
  import '#lib/components/styles/ux/loading.css';

  /** Owner's fix 3b: grey rows in the shape of what is coming, so the page doesn't jump when
   * it arrives. `card` is a feed card (header, title, two lines); `row` a list row (face and
   * two lines, as chats and people). */
  let { count = 3, kind = 'card' }: { count?: number; kind?: 'card' | 'row' } = $props();
</script>

<div class="skeleton-list" class:rows={kind === 'row'} aria-hidden="true">
  {#each { length: count }, i (i)}
    {#if kind === 'card'}
      <div class="skeleton-card">
        <div class="skeleton-head">
          <span class="sk sk-face"></span><span class="sk sk-line" style="width: 38%"></span>
        </div>
        <span class="sk sk-title" style={`width: ${72 - (i % 3) * 12}%`}></span>
        <span class="sk sk-line"></span>
        <span class="sk sk-line" style="width: 64%"></span>
      </div>
    {:else}
      <div class="skeleton-row">
        <span class="sk sk-face big"></span>
        <div>
          <span class="sk sk-line" style={`width: ${54 - (i % 3) * 8}%`}></span>
          <span class="sk sk-line" style="width: 78%"></span>
        </div>
      </div>
    {/if}
  {/each}
</div>

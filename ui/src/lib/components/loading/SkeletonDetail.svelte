<script lang="ts">
  import '#lib/components/styles/ux/loading.css';

  /**
   * A detail page while it loads (owner, 2026-11-18: the shapes sit where the content lands,
   * inside the page's margins): the tabs, the type tile and channel, the title, the author,
   * the facts card, the description, and the back button and dock in place of the nav.
   * Channel, community and feedback pages have no tabs or dock.
   */
  let { tabs = true, dock = true }: { tabs?: boolean; dock?: boolean } = $props();
</script>

<div class="skeleton-detail" aria-hidden="true">
  {#if tabs}
    <div class="skd-tabs">
      {#each [54, 66, 34, 38, 52] as width (width)}
        <span class="sk" style={`width: ${width}px`}></span>
      {/each}
    </div>
  {/if}
  <div class="skd-head">
    <span class="sk skd-tile"></span>
    <span class="skd-lines">
      <span class="sk skd-h" style="width: 38%"></span>
      <span class="sk skd-h small" style="width: 24%"></span>
    </span>
  </div>
  <span class="sk skd-title" style="width: 86%"></span>
  <div class="skd-author">
    <span class="sk skd-dot"></span><span class="sk skd-h" style="width: 46%"></span>
  </div>
  <div class="skd-card">
    <div class="skd-row">
      <span class="sk skd-icon"></span>
      <span class="skd-lines">
        <span class="sk skd-h" style="width: 34%"></span>
        <span class="sk skd-h small" style="width: 58%"></span>
      </span>
    </div>
    <div class="skd-row">
      <span class="sk skd-icon"></span><span class="sk skd-h" style="width: 40%"></span>
    </div>
  </div>
  <div class="skd-text">
    <span class="sk skd-h" style="width: 100%"></span>
    <span class="sk skd-h" style="width: 72%"></span>
  </div>
</div>
{#if dock}
  <div class="skd-dock" aria-hidden="true">
    <span class="sk skd-back"></span><span class="sk skd-capsule"></span>
  </div>
{/if}

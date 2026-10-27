<script lang="ts">
  import { railKicker, type RailItem } from '#lib/domain/activityRail.ts';

  /** web's Cleared section: dismissed cards, with Restore and Restore all. */
  interface Props {
    items: RailItem[];
    onOpen: (item: RailItem) => void;
    onRestore: (id: string) => void;
    onRestoreAll: () => void;
  }

  let p: Props = $props();
  let open = $state(false);
</script>

<details class="rail-section rail-section-cleared cleared-section" bind:open>
  <summary class="rail-section-summary">
    <span class="summary-label">
      <span class="chevron" aria-hidden="true"></span>
      <span>Cleared</span>
    </span>
  </summary>
  <div class="cleared-body">
    <button
      class="cleared-restore-all"
      type="button"
      onclick={() => {
        p.onRestoreAll();
        open = false;
      }}
    >
      Restore all
    </button>
    <div class="cleared-list">
      {#each p.items as item (item.id)}
        <div class="cleared-row">
          <button class="cleared-open-button" type="button" onclick={() => p.onOpen(item)}>
            <span class="cleared-kind">{railKicker(item)}</span>
            <strong>{item.title}</strong>
            {#if item.meta}
              <span class="cleared-meta">{item.meta}</span>
            {/if}
          </button>
          <button
            class="cleared-restore-one"
            type="button"
            onclick={(e) => {
              e.stopPropagation();
              p.onRestore(item.id);
            }}
          >
            Restore
          </button>
        </div>
      {/each}
    </div>
  </div>
</details>

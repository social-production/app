<script lang="ts">
  import AvatarBadge from '#lib/components/icons/AvatarBadge.svelte';
  import type { Contact } from '#lib/domain/messages.ts';

  /** web's SuggestionList with avatars: people matching what was typed. */
  interface Props {
    items: Contact[];
    query: string;
    emptyCopy: string;
    onSelect: (contact: Contact) => void;
  }

  let { items, query, emptyCopy, onSelect }: Props = $props();
</script>

{#if query.trim()}
  {#if items.length > 0}
    <div class="suggestion-list" role="listbox">
      {#each items as item (item.agent)}
        <button
          class="suggestion-row"
          role="option"
          aria-selected="false"
          type="button"
          onclick={() => onSelect(item)}
        >
          <AvatarBadge size="sm" name={item.name} />
          <span>{item.name}</span>
        </button>
      {/each}
    </div>
  {:else}
    <p class="empty-row">{emptyCopy}</p>
  {/if}
{/if}

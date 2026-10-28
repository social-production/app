<script lang="ts">
  import SuggestionList from '#lib/components/messages/SuggestionList.svelte';
  import type { Contact } from '#lib/domain/messages.ts';

  /** web's CreateEventAudienceSelector for people: chosen chips, then a search field. */
  interface Props {
    label: string;
    placeholder: string;
    helperText?: string;
    selected: Contact[];
    search: (query: string) => Promise<Contact[]>;
    onAdd: (contact: Contact) => void;
    onRemove: (agent: string) => void;
  }

  let { label, placeholder, helperText = '', selected, search, onAdd, onRemove }: Props = $props();

  let query = $state('');
  let found = $state<Contact[]>([]);
  let searchId = 0;
  const items = $derived(found.filter((c) => !selected.some((s) => s.agent === c.agent)));

  $effect(() => {
    const q = query.trim();
    const id = ++searchId;
    if (!q) {
      found = [];
      return;
    }
    search(q)
      .then((people) => id === searchId && (found = people))
      .catch(() => id === searchId && (found = []));
  });

  function add(contact: Contact): void {
    onAdd(contact);
    query = '';
  }

  // web's commitSingleSuggestion: Enter picks the only match.
  function onKeydown(event: KeyboardEvent): void {
    const [only, ...rest] = items;
    if (event.key === 'Enter' && only && rest.length === 0) {
      event.preventDefault();
      add(only);
    }
  }
</script>

<div class="audience-selector">
  <span class="field-label">{label}</span>
  <div class="token-input-stack">
    {#if selected.length > 0}
      <div class="selected-row" aria-label="Selected audience">
        {#each selected as item (item.agent)}
          <button class="selected-chip" type="button" onclick={() => onRemove(item.agent)}>
            <span>{item.name}</span>
            <span class="chip-remove" aria-hidden="true">×</span>
          </button>
        {/each}
      </div>
    {/if}
    <input bind:value={query} {placeholder} aria-label={label} onkeydown={onKeydown} />
    <SuggestionList {items} {query} emptyCopy="No matching people." onSelect={add} />
  </div>
  {#if helperText}
    <p class="helper-text">{helperText}</p>
  {/if}
</div>

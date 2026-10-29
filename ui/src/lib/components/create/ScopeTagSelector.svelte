<script lang="ts">
  /**
   * web's CreateScopeTagSelector: chosen channels or communities as chips, a search field and
   * its matches (web's SuggestionList). Enter picks the only match.
   */
  interface Item {
    key: string;
    label: string;
  }

  interface Props {
    label: string;
    placeholder: string;
    helperText?: string;
    selected: Item[];
    search: (query: string) => Promise<Item[]>;
    onAdd: (key: string) => void;
    onRemove: (key: string) => void;
  }

  let { label, placeholder, helperText = '', selected, search, onAdd, onRemove }: Props = $props();

  let query = $state('');
  let found = $state<Item[]>([]);
  let searchId = 0;
  const items = $derived(found.filter((f) => !selected.some((s) => s.key === f.key)));

  $effect(() => {
    const q = query.trim();
    const id = ++searchId;
    if (!q) {
      found = [];
      return;
    }
    search(q)
      .then((r) => id === searchId && (found = r))
      .catch(() => id === searchId && (found = []));
  });

  function add(key: string): void {
    onAdd(key);
    query = '';
  }

  function onKeydown(event: KeyboardEvent): void {
    const [only, ...rest] = items;
    if (event.key === 'Enter' && only && rest.length === 0) {
      event.preventDefault();
      add(only.key);
    }
  }
</script>

<div class="audience-selector">
  <span class="field-label">{label}</span>
  <div class="token-input-stack">
    {#if selected.length > 0}
      <div class="selected-row" aria-label="Selected tags">
        {#each selected as item (item.key)}
          <button class="selected-chip" type="button" onclick={() => onRemove(item.key)}>
            <span>{item.label}</span>
            <span class="chip-remove" aria-hidden="true">×</span>
          </button>
        {/each}
      </div>
    {/if}
    <input bind:value={query} {placeholder} aria-label={label} onkeydown={onKeydown} />
    {#if query.trim() && items.length > 0}
      <div class="suggestion-list" role="listbox">
        {#each items as item (item.key)}
          <button
            class="suggestion-row"
            role="option"
            aria-selected="false"
            type="button"
            onclick={() => add(item.key)}
          >
            <span>{item.label}</span>
          </button>
        {/each}
      </div>
    {/if}
  </div>
  {#if helperText}<p class="helper-text">{helperText}</p>{/if}
</div>

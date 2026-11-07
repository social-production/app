<script lang="ts">
  import { onMount, tick } from 'svelte';
  import { portal } from '#lib/components/actions/portal.ts';
  import {
    emptyPlaceValue,
    isPickSelected,
    onlinePlaceValue,
    typedPlaceValue,
    type PlaceMode,
    type PlaceQuickPick,
    type PlaceValue,
  } from '#lib/domain/places.ts';
  import '#lib/components/styles/location.css';

  /**
   * web's `components/shared/LocationPicker.svelte` (ADR 0080). The search comes in as a prop
   * (components don't call services); web's `persistOnSelect` has no counterpart, because the
   * app keeps the place on the record itself instead of a `locations` row.
   */
  interface Props {
    value: PlaceValue;
    onChange: (value: PlaceValue) => void;
    search: (query: string, limit: number) => Promise<PlaceValue[]>;
    /** A sentence for people, from the search's failure (rate limit vs. outage). */
    failure: (error: unknown) => string;
    modes?: PlaceMode[];
    compact?: boolean;
    elevated?: boolean;
    placeholder?: string;
    disabled?: boolean;
    portaled?: boolean;
    preferAbove?: boolean;
    preserveCoordsWhileEditing?: boolean;
    quickPicks?: PlaceQuickPick[];
  }

  let {
    value,
    onChange,
    search,
    failure,
    modes = ['physical', 'online'],
    compact = false,
    elevated = false,
    placeholder = 'Search a place',
    disabled = false,
    portaled = false,
    preferAbove = false,
    preserveCoordsWhileEditing = false,
    quickPicks = [],
  }: Props = $props();

  let query = $state('');
  let suggestions = $state<PlaceValue[]>([]);
  let searching = $state(false);
  let searchError = $state('');
  let listOpen = $state(false);
  let editing = false;
  let input = $state<HTMLInputElement>();
  let list = $state<HTMLUListElement>();
  let listStyle = $state('');
  let timer: ReturnType<typeof setTimeout> | undefined;
  let asked = 0;

  // web syncs the field to the value while the person isn't typing in it.
  $effect.pre(() => {
    const label = value.displayLabel;
    if (!editing) query = label;
  });

  async function position() {
    if (!portaled || !input || !list) return;
    await tick();
    const trigger = input.getBoundingClientRect();
    const menu = list.getBoundingClientRect();
    const left = Math.max(8, Math.min(trigger.left, window.innerWidth - trigger.width - 8));
    let top = trigger.top - menu.height - 4;
    if (top < 8) top = trigger.bottom + 4;
    top = Math.max(8, Math.min(top, window.innerHeight - menu.height - 8));
    listStyle = `top: ${top}px; left: ${left}px; width: ${trigger.width}px;`;
  }

  function runSearch(next: string) {
    clearTimeout(timer);
    const trimmed = next.trim();
    if (trimmed.length < 2) {
      suggestions = [];
      searching = false;
      return;
    }
    timer = setTimeout(() => void lookup(trimmed), 300);
  }

  async function lookup(trimmed: string) {
    const ticket = ++asked;
    searching = true;
    searchError = '';
    try {
      const found = await search(trimmed, 8);
      if (ticket !== asked) return;
      suggestions = found;
      listOpen = found.length > 0;
      if (listOpen) requestAnimationFrame(() => void position());
    } catch (error) {
      if (ticket !== asked) return;
      suggestions = [];
      listOpen = false;
      searchError = failure(error);
    } finally {
      if (ticket === asked) searching = false;
    }
  }

  function setValue(next: PlaceValue, syncQuery = true) {
    if (syncQuery) query = next.displayLabel;
    onChange(next);
  }

  function closeList() {
    editing = false;
    listOpen = false;
    suggestions = [];
  }

  function pick(item: PlaceValue) {
    closeList();
    setValue({ ...item, mode: item.isOnline ? 'online' : 'physical' });
  }

  function commitTyped(label: string) {
    const trimmed = label.trim();
    if (!trimmed) return;
    closeList();
    setValue({ ...typedPlaceValue(trimmed), precision: value.precision });
  }

  function onfocus() {
    editing = true;
    if (suggestions.length > 0) {
      listOpen = true;
      requestAnimationFrame(() => void position());
    }
  }

  function oninput() {
    editing = true;
    listOpen = false;
    runSearch(query);
    const trimmed = query.trim();
    if (value.mode === 'physical' && trimmed && !preserveCoordsWhileEditing) {
      setValue({ ...typedPlaceValue(trimmed), precision: value.precision }, false);
    }
  }

  function onblur() {
    if (listOpen || searching) return;
    editing = false;
    const trimmed = query.trim();
    if (trimmed === value.displayLabel.trim() && value.latitude != null) return;
    if (preserveCoordsWhileEditing) {
      if (trimmed) query = value.displayLabel;
      return;
    }
    setValue({ ...typedPlaceValue(trimmed), precision: value.precision });
  }

  function onkeydown(event: KeyboardEvent) {
    if (event.key !== 'Enter') return;
    const first = suggestions[0];
    if (listOpen && first) {
      event.preventDefault();
      pick(first);
    } else if (query.trim()) {
      event.preventDefault();
      commitTyped(query);
    }
  }

  function selectMode(mode: PlaceMode) {
    editing = false;
    if (mode === 'online') setValue(onlinePlaceValue());
    else if (mode === 'tbd') setValue({ ...emptyPlaceValue('tbd'), displayLabel: 'To be decided' });
    else setValue(emptyPlaceValue('physical'));
  }

  function selectQuickPick(choice: PlaceQuickPick) {
    closeList();
    setValue({ ...choice.value, mode: 'physical', isOnline: false });
  }

  onMount(() => {
    const reposition = () => {
      if (listOpen && document.activeElement === input) void position();
    };
    window.addEventListener('resize', reposition);
    window.addEventListener('scroll', reposition, true);
    return () => {
      clearTimeout(timer);
      window.removeEventListener('resize', reposition);
      window.removeEventListener('scroll', reposition, true);
    };
  });
</script>

{#snippet suggestionList()}
  <ul
    bind:this={list}
    class="suggestions"
    class:portaled
    class:prefer-above={preferAbove && !portaled}
    style={portaled ? listStyle : undefined}
  >
    {#each suggestions as suggestion (suggestion.providerPlaceId ?? suggestion.displayLabel)}
      <li>
        <button
          type="button"
          onmousedown={(e) => e.preventDefault()}
          onclick={() => pick(suggestion)}
        >
          {suggestion.displayLabel}
        </button>
      </li>
    {/each}
  </ul>
{/snippet}

<div class="location-picker" class:compact class:elevated>
  {#if modes.length > 1}
    <div class="mode-row" role="group" aria-label="Location type">
      {#each modes as mode (mode)}
        <button
          class="mode-chip"
          class:active={value.mode === mode}
          {disabled}
          type="button"
          onclick={() => selectMode(mode)}
        >
          {mode === 'physical' ? 'Physical' : mode === 'online' ? 'Online' : 'TBD'}
        </button>
      {/each}
    </div>
  {/if}

  {#if value.mode === 'physical'}
    <div class="search-wrap">
      <input
        bind:this={input}
        {disabled}
        {placeholder}
        aria-label="Place"
        bind:value={query}
        autocomplete="off"
        {onfocus}
        {oninput}
        {onblur}
        {onkeydown}
      />
      {#if searching}<span class="status">Searching…</span>{/if}
      {#if searchError}<p class="error" role="alert">{searchError}</p>{/if}
      {#if !compact && query.trim() && query.trim() !== value.displayLabel.trim()}
        <button class="use-typed-name" type="button" onclick={() => commitTyped(query)}>
          Use “{query.trim()}” as the place name
        </button>
      {/if}
      {#if listOpen && suggestions.length > 0}
        {#if portaled}
          <!-- Portaled lists keep the root class so the ported styles still reach them. -->
          <div class="location-picker location-portal" use:portal>{@render suggestionList()}</div>
        {:else}
          {@render suggestionList()}
        {/if}
      {/if}
    </div>
    {#if quickPicks.length > 0}
      <div
        class="quick-picks"
        class:compact-picks={elevated}
        role="group"
        aria-label="Suggested locations"
      >
        {#each quickPicks.slice(0, elevated ? 3 : quickPicks.length) as choice (choice.id)}
          <button
            class="quick-pick"
            class:active={isPickSelected(choice, value)}
            {disabled}
            type="button"
            onclick={() => selectQuickPick(choice)}
          >
            <span class="quick-pick-label">{choice.label}</span>
            <span class="quick-pick-source">{choice.sourceLabel}</span>
          </button>
        {/each}
      </div>
    {/if}
  {:else if value.mode !== 'online'}
    <p class="tbd-label">Location will be decided later.</p>
  {/if}
</div>

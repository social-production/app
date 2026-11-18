<script lang="ts">
  import { t } from '#lib/i18n/index.ts';
  import ToolbarIcon from '#lib/components/icons/ToolbarIcon.svelte';
  import type { PlaceSearch, PlaceValue } from '#lib/domain/places.ts';

  import '#lib/components/styles/ux/regionsettings.css';

  /**
   * Owner's fix 3d (2026-11-17): the place sheet keeps its field and list still while results
   * update. "Use my location" comes first, and each match shows its area under the name.
   * `picked` is the place the sheet's button will save.
   */
  interface Props extends PlaceSearch {
    initial: string;
    picked: PlaceValue | null;
    onPick: (place: PlaceValue) => void;
    onLocate: () => void;
  }

  let { initial, picked, onPick, onLocate, search, failure }: Props = $props();

  // svelte-ignore state_referenced_locally
  let query = $state(initial);
  let results = $state<PlaceValue[]>([]);
  let message = $state('');
  let timer: ReturnType<typeof setTimeout> | null = null;
  let asked = 0;

  function onInput(value: string): void {
    query = value;
    if (timer) clearTimeout(timer);
    if (value.trim().length < 2) {
      message = '';
      return; // the last results stay put until new ones arrive
    }
    timer = setTimeout(() => void run(value.trim()), 250);
  }

  async function run(text: string): Promise<void> {
    const mine = ++asked;
    try {
      const found = await search(text, 6);
      if (mine !== asked) return;
      results = found;
      message = found.length ? '' : 'No places match yet.';
    } catch (e) {
      if (mine === asked) message = failure(e);
    }
  }

  const area = (p: PlaceValue): string => [p.region, p.country].filter(Boolean).join(', ');
  const name = (p: PlaceValue): string => p.displayLabel.split(',')[0] ?? p.displayLabel;
</script>

<div class="place-search">
  <label class="place-search-field">
    <ToolbarIcon name="search" />
    <input
      aria-label={t('Search a place')}
      placeholder={t('Search a place')}
      value={query}
      oninput={(e) => onInput(e.currentTarget.value)}
    />
  </label>
  <ul class="place-search-list">
    <li>
      <button type="button" onclick={onLocate}>
        <ToolbarIcon name="locate" />
        <span
          ><strong>{t('Use my location')}</strong><small>{t('Ask this device once')}</small></span
        >
      </button>
    </li>
    {#each results as place, i (i)}
      <li>
        <button
          type="button"
          class:picked={picked?.providerPlaceId === place.providerPlaceId &&
            picked?.displayLabel === place.displayLabel}
          onclick={() => onPick(place)}
        >
          <ToolbarIcon name="map-pin" />
          <span><strong>{name(place)}</strong><small>{area(place)}</small></span>
        </button>
      </li>
    {/each}
  </ul>
  <p class="place-search-note">{message}</p>
</div>

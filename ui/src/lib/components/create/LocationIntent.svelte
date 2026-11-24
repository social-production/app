<script lang="ts">
  import { t } from '#lib/i18n/index.ts';
  import LocationPicker from '#lib/components/location/LocationPicker.svelte';
  import ChoiceRows from './ChoiceRows.svelte';
  import type { PlaceValue } from '#lib/domain/places.ts';
  import type { LocationChoice } from '#lib/domain/projects.ts';

  /** web's CreateLocationIntent, with web's LocationPicker for a physical place (ADR 0080). */
  interface Props {
    intent: LocationChoice;
    place: PlaceValue;
    onIntent: (intent: LocationChoice) => void;
    onPlace: (place: PlaceValue) => void;
    /** The gateway's place search and its failure sentence (features pass the service). */
    search: (query: string, limit: number) => Promise<PlaceValue[]>;
    failure: (error: unknown) => string;
    /** web's `allowLater` and `helperText` (help requests need a place now). */
    allowLater?: boolean;
    helperText?: string;
  }

  let {
    intent,
    place,
    onIntent,
    onPlace,
    search,
    failure,
    allowLater = true,
    helperText = t('People nearby find it on the map.'),
  }: Props = $props();

  // Three rows, each with one line that says what the choice means.
  const choices: { value: LocationChoice; label: string; hint: string }[] = [
    { value: 'physical', label: t('In person'), hint: '' },
    { value: 'online', label: t('Online'), hint: t('It will not show on the map.') },
    {
      value: 'later',
      label: t('Not yet'),
      hint: t('Add a place later. Until then it is not on the map.'),
    },
  ];
  const shown = $derived(choices.filter((c) => allowLater || c.value !== 'later'));
  // A pin, a globe and a clock.
  const ICONS: Record<LocationChoice, string> = {
    physical:
      'M12 21s-6-5.6-6-11a6 6 0 0 1 12 0c0 5.4-6 11-6 11ZM12 12.5a2.5 2.5 0 1 0 0-5 2.5 2.5 0 0 0 0 5Z',
    online:
      'M12 21a9 9 0 1 0 0-18 9 9 0 0 0 0 18ZM3 12h18M12 3c2.5 2.5 3.5 5.5 3.5 9s-1 6.5-3.5 9M12 3c-2.5 2.5-3.5 5.5-3.5 9s1 6.5 3.5 9',
    later: 'M12 21a9 9 0 1 0 0-18 9 9 0 0 0 0 18ZM12 7v5l3 2',
  };
</script>

<div class="location-intent">
  <ChoiceRows
    value={intent}
    ariaLabel={t('Location intent')}
    options={shown.map((c) => ({
      value: c.value,
      label: c.label,
      hint: c.value === 'physical' ? helperText || t('On the map, at a place you pick.') : c.hint,
      icon: ICONS[c.value],
    }))}
    onChange={onIntent}
  >
    {#snippet under(chosen)}
      {#if chosen === 'physical'}
        <LocationPicker
          value={place}
          modes={['physical']}
          placeholder={t('Search a place')}
          onChange={onPlace}
          {search}
          {failure}
        />
      {/if}
    {/snippet}
  </ChoiceRows>
</div>

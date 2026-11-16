<script lang="ts">
  import LocationPicker from '#lib/components/location/LocationPicker.svelte';
  import SegmentedChoice from '#lib/components/SegmentedChoice.svelte';
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
    helperText = 'People nearby will find it on the map. You can change the place later.',
  }: Props = $props();

  // Polish Pass mockups: three words in one segmented control, and one line that says
  // what the choice means.
  const choices: { value: LocationChoice; label: string; hint: string }[] = [
    { value: 'physical', label: 'In person', hint: '' },
    { value: 'online', label: 'Online', hint: 'It will not show on the map.' },
    {
      value: 'later',
      label: 'Not yet',
      hint: 'Add a place later. Until then it is not on the map.',
    },
  ];
  const shown = $derived(choices.filter((c) => allowLater || c.value !== 'later'));
  const hint = $derived(choices.find((c) => c.value === intent)?.hint || helperText);
</script>

<div class="location-intent">
  <SegmentedChoice
    value={intent}
    ariaLabel="Location intent"
    options={shown.map((c) => ({ value: c.value, label: c.label }))}
    onChange={onIntent}
  />
  {#if intent === 'physical'}
    <LocationPicker
      value={place}
      modes={['physical']}
      placeholder="Search a place"
      onChange={onPlace}
      {search}
      {failure}
    />
  {/if}
  <p class="helper">{hint}</p>
</div>

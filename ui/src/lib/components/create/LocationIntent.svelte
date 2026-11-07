<script lang="ts">
  import LocationPicker from '#lib/components/location/LocationPicker.svelte';
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
    helperText = 'A physical location helps people discover this on the map. You can change it later.',
  }: Props = $props();

  const choices: { value: LocationChoice; title: string; hint: string }[] = [
    {
      value: 'physical',
      title: 'Physical',
      hint: 'Discoverable on the map once coordinates are set.',
    },
    { value: 'online', title: 'Online', hint: 'Not shown as a map pin.' },
    {
      value: 'later',
      title: 'Decide later',
      hint: 'Create now, add a place later. Not map-discoverable yet.',
    },
  ];
</script>

<div class="location-intent">
  <p class="helper">{helperText}</p>
  <div class="intent-row" role="radiogroup" aria-label="Location intent">
    {#each choices.filter((c) => allowLater || c.value !== 'later') as choice (choice.value)}
      <label class="intent-option">
        <input
          type="radio"
          name="location-intent"
          value={choice.value}
          checked={intent === choice.value}
          onchange={() => onIntent(choice.value)}
        />
        <span><strong>{choice.title}</strong><span class="hint">{choice.hint}</span></span>
      </label>
    {/each}
  </div>
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
</div>

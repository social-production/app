<script lang="ts">
  import type { LocationChoice } from '#lib/domain/projects.ts';

  /**
   * web's CreateLocationIntent. The place is a typed label until the map lands (M6), when
   * web's LocationPicker replaces the text field.
   */
  interface Props {
    intent: LocationChoice;
    place: string;
    onIntent: (intent: LocationChoice) => void;
    onPlace: (place: string) => void;
  }

  let { intent, place, onIntent, onPlace }: Props = $props();

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
  <p class="helper">
    A physical location helps people discover this on the map. You can change it later.
  </p>
  <div class="intent-row" role="radiogroup" aria-label="Location intent">
    {#each choices as choice (choice.value)}
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
    <input
      aria-label="Place"
      maxlength="160"
      placeholder="Search a place"
      value={place}
      oninput={(e) => onPlace(e.currentTarget.value)}
    />
  {/if}
</div>

<script lang="ts">
  import { t } from '#lib/i18n/index.ts';
  import ChoiceRows from '#lib/components/create/ChoiceRows.svelte';
  import LocationPicker from '#lib/components/location/LocationPicker.svelte';
  import { withLocation, type ActivityForm } from '#lib/domain/activityForm.ts';
  import { emptyPlaceValue, type PlaceQuickPick, type PlaceSearch } from '#lib/domain/places.ts';

  /** Step 2 (Activity creation mockup): the plan's place first, somewhere else, or online;
   * then which plan stage it is part of, as taps. */
  interface Props {
    form: ActivityForm;
    quickPicks: PlaceQuickPick[];
    places: PlaceSearch;
    stages: { id: string; label: string }[];
    onChange: (form: ActivityForm) => void;
  }

  let { form, quickPicks, places, stages, onChange }: Props = $props();

  type Where = 'usual' | 'elsewhere' | 'online';
  const usual = $derived(quickPicks[0] ?? null);
  let chosen = $state<Where | null>(null);
  const where = $derived<Where>(
    chosen ??
      (form.online
        ? 'online'
        : usual && (!form.place.trim() || form.place === usual.label)
          ? 'usual'
          : 'elsewhere'),
  );
  // Opening on the usual place fills it in.
  $effect(() => {
    if (where === 'usual' && usual && form.place !== usual.label)
      onChange(withLocation(form, usual.value));
  });

  const PIN =
    'M12 21s-6-5.6-6-11a6 6 0 0 1 12 0c0 5.4-6 11-6 11ZM12 12.5a2.5 2.5 0 1 0 0-5 2.5 2.5 0 0 0 0 5Z';
  const SEARCH = 'M11 17a6 6 0 1 0 0-12 6 6 0 0 0 0 12ZM20 20l-4.5-4.5';
  const GLOBE =
    'M12 21a9 9 0 1 0 0-18 9 9 0 0 0 0 18ZM3 12h18M12 3c2.5 2.5 3.5 5.5 3.5 9s-1 6.5-3.5 9M12 3c-2.5 2.5-3.5 5.5-3.5 9s1 6.5 3.5 9';
  const options = $derived([
    ...(usual
      ? [{ value: 'usual' as Where, label: t('The usual place'), hint: usual.label, icon: PIN }]
      : []),
    {
      value: 'elsewhere' as Where,
      label: usual ? t('Somewhere else') : t('In person'),
      hint: t('Search a place'),
      icon: SEARCH,
    },
    { value: 'online' as Where, label: t('Online'), hint: t('Add a meeting link'), icon: GLOBE },
  ]);

  function choose(next: Where): void {
    chosen = next;
    if (next === 'usual' && usual) onChange(withLocation(form, usual.value));
    else if (next === 'online')
      onChange(withLocation(form, { ...emptyPlaceValue('online'), displayLabel: 'Online' }));
    else onChange(withLocation(form, emptyPlaceValue()));
  }
</script>

<ChoiceRows value={where} ariaLabel={t('Where')} {options} onChange={choose}>
  {#snippet under(v)}
    {#if v === 'elsewhere'}
      <LocationPicker
        value={form.location}
        portaled
        modes={['physical']}
        placeholder={t('Search a place')}
        onChange={(value) => onChange(withLocation(form, value))}
        search={places.search}
        failure={places.failure}
      />
    {:else if v === 'online'}
      <label class="activity-field">
        <span>{t('Meeting link')}</span>
        <input
          value={form.meetingUrl}
          type="url"
          inputmode="url"
          maxlength="500"
          placeholder="https://"
          oninput={(e) => onChange({ ...form, meetingUrl: e.currentTarget.value })}
        />
      </label>
    {/if}
  {/snippet}
</ChoiceRows>
{#if stages.length > 0}
  <div class="activity-stages">
    <span>{t('Part of')}</span>
    <div class="activity-chips" role="radiogroup" aria-label={t('Plan stage')}>
      {#each stages as stage (stage.id)}
        <button
          type="button"
          role="radio"
          aria-checked={form.planStage === stage.id}
          class:sel={form.planStage === stage.id}
          onclick={() => onChange({ ...form, planStage: stage.id })}>{t(stage.label)}</button
        >
      {/each}
    </div>
  </div>
{/if}

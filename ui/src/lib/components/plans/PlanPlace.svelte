<script lang="ts">
  import { t } from '#lib/i18n/index.ts';
  import ChoiceRows from '#lib/components/create/ChoiceRows.svelte';
  import LocationPicker from '#lib/components/location/LocationPicker.svelte';
  import { onlinePlaceValue, type PlaceSearch } from '#lib/domain/places.ts';
  import {
    sameAsProduction,
    withPlanPlace,
    type PlanForm,
    type ProductionLocation,
  } from '#lib/domain/planForm.ts';

  /**
   * Where a plan happens, as rows (owner, 2026-11-24: the In person / Online switcher "looks
   * like the old way"): the production plan's place when there is one, In person with a place
   * search under it, or Online.
   */
  interface Props {
    form: PlanForm;
    places: PlaceSearch;
    /** Distribution and access plans can reuse the production plan's place. */
    production?: ProductionLocation | null;
  }

  let { form = $bindable(), places, production = null }: Props = $props();

  type Where = 'same' | 'physical' | 'online';
  const PIN =
    'M12 21s-6-5.6-6-11a6 6 0 0 1 12 0c0 5.4-6 11-6 11ZM12 12.5a2.5 2.5 0 1 0 0-5 2.5 2.5 0 0 0 0 5Z';
  const GLOBE =
    'M12 21a9 9 0 1 0 0-18 9 9 0 0 0 0 18ZM3 12h18M12 3c2.5 2.5 3.5 5.5 3.5 9s-1 6.5-3.5 9M12 3c-2.5 2.5-3.5 5.5-3.5 9s1 6.5 3.5 9';
  const SAME = 'M9 12l2 2 4-4M12 21a9 9 0 1 0 0-18 9 9 0 0 0 0 18Z';

  const where = $derived<Where>(
    form.sameAsProduction ? 'same' : form.place.mode === 'online' ? 'online' : 'physical',
  );
  const options = $derived([
    ...(production
      ? [
          {
            value: 'same' as Where,
            label: t('Same as the production plan'),
            hint: production.label,
            icon: SAME,
          },
        ]
      : []),
    {
      value: 'physical' as Where,
      label: t('In person'),
      hint: t('On the map, at a place you pick.'),
      icon: PIN,
    },
    { value: 'online' as Where, label: t('Online'), hint: t('Not on the map.'), icon: GLOBE },
  ]);

  function choose(next: Where): void {
    if (next === 'same') {
      form = sameAsProduction(form, true, production);
      return;
    }
    const base = form.sameAsProduction ? sameAsProduction(form, false, production) : form;
    form = withPlanPlace(
      base,
      next === 'online' ? onlinePlaceValue() : { ...base.place, mode: 'physical' },
    );
  }
</script>

<ChoiceRows value={where} ariaLabel={t('Where')} {options} onChange={choose}>
  {#snippet under(chosen)}
    {#if chosen === 'physical'}
      <LocationPicker
        value={form.place}
        modes={['physical']}
        placeholder={t('Search a place')}
        onChange={(value) => (form = withPlanPlace(form, value))}
        search={places.search}
        failure={places.failure}
      />
    {/if}
  {/snippet}
</ChoiceRows>

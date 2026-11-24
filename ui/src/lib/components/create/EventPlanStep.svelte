<script lang="ts">
  import { t } from '#lib/i18n/index.ts';
  import { displayZone } from '#lib/domain/zone.ts';
  import { zoneOf } from '#lib/domain/activityTime.ts';
  import type { PlanForm } from '#lib/domain/planForm.ts';
  import PlanStagesEditor from '#lib/components/plans/PlanStagesEditor.svelte';

  import ChoiceRows from './ChoiceRows.svelte';
  import RequiredLabel from './RequiredLabel.svelte';

  import '#lib/components/styles/ux/planstages.css';

  /**
   * web's create-event Plan step for organizer-controlled private events: the plan, its
   * schedule and its stages, set up front so the event starts in Activity (ADR 0061), in the
   * plan wizard's look (choice rows and stage cards, owner 2026-11-24).
   */
  interface Props {
    plan: PlanForm;
    problems: string[];
  }

  let { plan = $bindable(), problems }: Props = $props();

  const zone = displayZone();
</script>

<div class="form-stack plan-step">
  <p class="helper-text">
    {t(
      'Organizer-controlled private events skip signals, values, and proposal votes. Set the plan now so the event can start in Activity. Members can still join and sign up for roles once activities exist.',
    )}
  </p>
  <label>
    <span class="field-inline-label"><RequiredLabel>{t('Plan title')}</RequiredLabel></span>
    <input bind:value={plan.title} aria-required="true" />
  </label>
  <label>
    <span class="field-inline-label"><RequiredLabel>{t('Plan description')}</RequiredLabel></span>
    <textarea bind:value={plan.description} rows="3" aria-required="true"></textarea>
  </label>
  <span class="field-inline-label">{t('When is it?')}</span>
  <ChoiceRows
    value={plan.scheduleMode}
    ariaLabel={t('Schedule')}
    options={[
      {
        value: 'date',
        label: t('Single date'),
        hint: t('One day, with a start and finish time.'),
        icon: 'M5 5h14v15H5zM5 10h14M9 3v4M15 3v4',
      },
      {
        value: 'range',
        label: t('Date range'),
        hint: t('Several days in a row, like a festival.'),
        icon: 'M4 6h11v13H4zM4 10h11M8 4v3M18 9h2v12H9v-2',
      },
    ]}
    onChange={(value) => (plan.scheduleMode = value)}
  >
    {#snippet under(mode)}
      <div class="schedule-fields is-range">
        {#if mode === 'date'}
          <label>
            <span class="field-inline-label"><RequiredLabel>{t('Date')}</RequiredLabel></span>
            <input type="date" bind:value={plan.scheduledDate} />
          </label>
          <span></span>
        {:else}
          <label>
            <span class="field-inline-label"><RequiredLabel>{t('Start date')}</RequiredLabel></span>
            <input type="date" bind:value={plan.rangeStartDate} />
          </label>
          <label>
            <span class="field-inline-label"><RequiredLabel>{t('End date')}</RequiredLabel></span>
            <input type="date" bind:value={plan.rangeEndDate} />
          </label>
        {/if}
        <label>
          <span class="field-inline-label"><RequiredLabel>{t('Start time')}</RequiredLabel></span>
          <input type="time" bind:value={plan.startTime} />
        </label>
        <label>
          <span class="field-inline-label"><RequiredLabel>{t('Finish time')}</RequiredLabel></span>
          <input type="time" bind:value={plan.finishTime} />
        </label>
      </div>
      <p class="helper-text">
        Times are in {zoneOf(new Date()) ? `${zoneOf(new Date())} (${zone})` : zone}.
      </p>
    {/snippet}
  </ChoiceRows>
  <span class="field-inline-label">{t('Stages')}</span>
  <PlanStagesEditor bind:stages={plan.stages} dated />
  {#if problems.length > 0}
    <ul class="validation-list">
      {#each problems as message (message)}
        <li>{message}</li>
      {/each}
    </ul>
  {/if}
</div>

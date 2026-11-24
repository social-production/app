<script lang="ts">
  import { t, plural } from '#lib/i18n/index.ts';
  import { displayZone } from '#lib/domain/zone.ts';
  import {
    SOFTWARE_LICENSE_BODY,
    SOFTWARE_LICENSE_LABEL,
    SOFTWARE_LICENSE_TITLE,
    SOFTWARE_LICENSE_WHY,
  } from '#lib/domain/policy.ts';
  import { zoneOf } from '#lib/domain/activityTime.ts';
  import { scheduleProblem } from '#lib/domain/eventSchedule.ts';
  import { type PlanForm, type PlanStep, type ProductionLocation } from '#lib/domain/planForm.ts';
  import type { PlaceSearch } from '#lib/domain/places.ts';
  import ChoiceRows from '#lib/components/create/ChoiceRows.svelte';
  import { REQUEST_MODES, type ProjectSubtype } from '#lib/domain/projects.ts';

  import RequiredLabel from '#lib/components/create/RequiredLabel.svelte';

  import PlanPlace from './PlanPlace.svelte';
  import PlanStagesEditor from './PlanStagesEditor.svelte';

  /**
   * The fields of one plan wizard step (web's PlanCreationWizard body) in the create flow's
   * look (Edit, Plan and Meetup mockups §B): choices as rows, labelled filled fields.
   */
  interface Props {
    step: PlanStep;
    form: PlanForm;
    subtypes: { value: ProjectSubtype; label: string }[];
    /** The approved production plan's location, for "Same as production plan location". */
    productionLocation: ProductionLocation | null;
    places: PlaceSearch;
    demand: number;
    opposition: number;
    required: number;
    members: number;
    /** Event plans give stages dates; other plans leave timing to activities. */
    dated: boolean;
  }

  let {
    step,
    form = $bindable(),
    subtypes,
    productionLocation,
    places,
    demand,
    opposition,
    required,
    members,
    dated,
  }: Props = $props();

  const total = $derived(demand + opposition);
  const zone = displayZone();
  const SUBTYPE_HINTS: Partial<Record<ProjectSubtype, string>> = {
    standard: t('Made, grown or done in person.'),
    software: t('Code in a public repository, under AGPL v3.'),
  };
  // A box, code brackets, a calendar and two calendars.
  const CUBE = 'M12 3 4 7.5v9L12 21l8-4.5v-9L12 3ZM4 7.5l8 4.5 8-4.5M12 12v9';
  const CODE = 'M8 8l-4 4 4 4M16 8l4 4-4 4M14 5l-4 14';
  const CALENDAR = 'M5 5h14v15H5zM5 10h14M9 3v4M15 3v4';
  const RANGE = 'M4 6h11v13H4zM4 10h11M8 4v3M18 9h2v12H9v-2';
</script>

{#if step.id === 'plan-overview'}
  <div class="overview-stack">
    <label>
      <span class="field-inline-label"><RequiredLabel>{t('Title')}</RequiredLabel></span>
      <input
        bind:value={form.title}
        maxlength="120"
        placeholder={t('Saturday veg share')}
        aria-required="true"
      />
    </label>
    <label>
      <span class="field-inline-label"><RequiredLabel>{t('Description')}</RequiredLabel></span>
      <textarea
        bind:value={form.description}
        rows="4"
        placeholder={t('Who it reaches, how often, and how it happens.')}
        aria-required="true"></textarea>
    </label>
  </div>
{:else if step.id === 'subtype'}
  <ChoiceRows
    value={form.subtype}
    ariaLabel={t('Plan type')}
    options={subtypes.map((o) => ({
      value: o.value,
      label: t(o.label),
      hint: SUBTYPE_HINTS[o.value],
      icon: o.value === 'software' ? CODE : CUBE,
    }))}
    onChange={(value) => (form.subtype = value)}
  />
{:else if step.id === 'repository'}
  <label>
    <span class="field-inline-label"><RequiredLabel>{t('Repository URL')}</RequiredLabel></span>
    <input
      bind:value={form.repositoryUrl}
      aria-required="true"
      inputmode="url"
      placeholder="https://github.com/org/repo"
    />
  </label>
  <details class="plan-license">
    <summary
      ><b>{SOFTWARE_LICENSE_LABEL}</b><span>{SOFTWARE_LICENSE_TITLE}</span><u>{t('Read')}</u
      ></summary
    >
    <p>{SOFTWARE_LICENSE_BODY}</p>
    <ul>
      {#each SOFTWARE_LICENSE_WHY as reason (reason)}<li>{reason}</li>{/each}
    </ul>
  </details>
{:else if step.id === 'location'}
  <PlanPlace bind:form {places} />
{:else if step.id === 'distribution-location'}
  <div class="settings-stack location-step">
    <PlanPlace bind:form {places} production={productionLocation} />
    <label class="recipients-field">
      <span class="field-inline-label">{t('Recipients')}</span>
      <textarea
        bind:value={form.recipients}
        rows="2"
        maxlength="500"
        placeholder={t('Who receives what this project makes?')}></textarea>
    </label>
  </div>
{:else if step.id === 'requests'}
  <ChoiceRows
    value={form.requests}
    ariaLabel={t('Requests')}
    options={[
      {
        value: '' as const,
        label: t('No requests for now'),
        hint: t('Members can turn them on later.'),
      },
      ...REQUEST_MODES.map((o) => ({ value: o.value, label: t(o.label), hint: t(o.summary) })),
    ]}
    onChange={(value) => (form.requests = value)}
  />
{:else if step.id === 'demand-note'}
  <div class="plan-signals" aria-label={t('Current support signals')}>
    <span><b>{demand}</b>{t('support')}</span>
    <span><b>{opposition}</b>{t('against')}</span>
    <span><b>{total === 0 ? 0 : Math.round((demand / total) * 100)}%</b>{t('share')}</span>
  </div>
  <label>
    <span class="field-inline-label"><RequiredLabel>{t('Support it answers')}</RequiredLabel></span>
    <textarea
      bind:value={form.demandNote}
      rows="4"
      aria-required="true"
      placeholder={t('Covers the 3 who asked, weekly.')}></textarea>
  </label>
  <p class="helper-copy">
    {t('Approval needs {n} of {members} to vote, and at least 66% Yes.', {
      n: required,
      members: plural(members, '{n} member', '{n} members'),
    })}
  </p>
{:else if step.id === 'schedule-mode'}
  <ChoiceRows
    value={form.scheduleMode}
    ariaLabel={t('Schedule')}
    options={[
      {
        value: 'date',
        label: t('Single date'),
        hint: t('One day, with a start and finish time.'),
        icon: CALENDAR,
      },
      {
        value: 'range',
        label: t('Date range'),
        hint: t('Several days in a row, like a festival.'),
        icon: RANGE,
      },
    ]}
    onChange={(value) => (form.scheduleMode = value)}
  />
{:else if step.id === 'schedule-date'}
  <label>
    <span class="field-inline-label">{t('Date')}</span>
    <input type="date" bind:value={form.scheduledDate} aria-label={t('Date')} />
  </label>
{:else if step.id === 'schedule-range'}
  <div class="field-grid">
    <label>
      <span>{t('Start date')}</span>
      <input type="date" bind:value={form.rangeStartDate} />
    </label>
    <label>
      <span>{t('End date')}</span>
      <input type="date" bind:value={form.rangeEndDate} />
    </label>
  </div>
{:else if step.id === 'schedule-time'}
  <div class="field-grid">
    <label>
      <span>{t('Start time')}</span>
      <input type="time" bind:value={form.startTime} />
    </label>
    <label>
      <span>{t('Finish time')}</span>
      <input type="time" bind:value={form.finishTime} />
    </label>
  </div>
  {#if form.startTime && form.finishTime && scheduleProblem(form)}
    <p class="helper-copy">{scheduleProblem(form)}</p>
  {/if}
  <!-- web's TimeZoneNote, without its Settings link: the app has no display time zone yet. -->
  <p class="helper-copy">
    Times are in {zoneOf(new Date()) ? `${zoneOf(new Date())} (${zone})` : zone}.
  </p>
{:else if step.id === 'stages'}
  <PlanStagesEditor bind:stages={form.stages} {dated} />
{/if}

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
  import {
    isMeetupForm,
    sameAsProduction,
    withPlanPlace,
    type PlanForm,
    type PlanStep,
    type ProductionLocation,
  } from '#lib/domain/planForm.ts';
  import { onlinePlaceValue, type PlaceSearch } from '#lib/domain/places.ts';
  import LocationIntent from '#lib/components/create/LocationIntent.svelte';
  import { REQUEST_MODES, type ProjectSubtype } from '#lib/domain/projects.ts';

  import MeetupSchedule from '#lib/components/create/MeetupSchedule.svelte';
  import RequiredLabel from '#lib/components/create/RequiredLabel.svelte';

  import PlanStagesEditor from './PlanStagesEditor.svelte';

  /**
   * The fields of one plan wizard step (web's PlanCreationWizard body), places through web's
   * LocationPicker (ADR 0080).
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
  }: Props = $props();

  const total = $derived(demand + opposition);
  const zone = displayZone();
</script>

{#snippet where()}
  <LocationIntent
    intent={form.place.mode === 'online' ? 'online' : 'physical'}
    place={form.place}
    allowLater={false}
    helperText=""
    search={places.search}
    failure={places.failure}
    onIntent={(next) =>
      (form = withPlanPlace(
        form,
        next === 'online' ? onlinePlaceValue() : { ...form.place, mode: 'physical' },
      ))}
    onPlace={(value) => (form = withPlanPlace(form, value))}
  />
{/snippet}

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
  <div class="plan-opts" role="radiogroup" aria-label={t('Plan type')}>
    {#each subtypes as option (option.value)}
      <label
        ><input type="radio" bind:group={form.subtype} value={option.value} /><b
          >{t(option.label)}</b
        ></label
      >
    {/each}
  </div>
{:else if step.id === 'repository'}
  <aside class="license-notice" aria-label={t('Software license guidance')}>
    <strong>License: {SOFTWARE_LICENSE_LABEL}</strong>
    <p>{SOFTWARE_LICENSE_TITLE}</p>
    <p>{SOFTWARE_LICENSE_BODY}</p>
    <ul>
      {#each SOFTWARE_LICENSE_WHY as reason (reason)}<li>{reason}</li>{/each}
    </ul>
  </aside>
  <input
    bind:value={form.repositoryUrl}
    aria-label={t('Repository URL')}
    placeholder="https://github.com/org/repo"
  />
{:else if step.id === 'location'}
  <div class="location-step">
    {@render where()}
  </div>
{:else if step.id === 'distribution-location'}
  <div class="settings-stack location-step">
    {#if productionLocation}
      <label class="checkbox-row">
        <input
          checked={form.sameAsProduction}
          type="checkbox"
          onchange={(e) =>
            (form = sameAsProduction(form, e.currentTarget.checked, productionLocation))}
        />
        <span>Same as production plan location ({t(productionLocation.label)})</span>
      </label>
    {/if}
    {#if !form.sameAsProduction}
      {@render where()}
    {/if}
    <label class="recipients-field">
      <span>{t('Recipients')}</span>
      <textarea
        bind:value={form.recipients}
        rows="2"
        maxlength="500"
        placeholder={t('Who receives what this project makes?')}></textarea>
    </label>
  </div>
{:else if step.id === 'requests'}
  <div class="plan-opts" role="radiogroup" aria-label={t('Requests')}>
    <label
      ><input type="radio" bind:group={form.requests} value="" /><b>{t('No requests for now')}</b
      ></label
    >
    {#each REQUEST_MODES as option (option.value)}
      <label
        ><input type="radio" bind:group={form.requests} value={option.value} /><b
          >{t(option.label)}</b
        ><small>{t(option.summary)}</small></label
      >
    {/each}
  </div>
{:else if step.id === 'demand-note'}
  <div class="signal-card" aria-label={t('Current support signals')}>
    <span class="signal-label">{t('Current support signals')}</span>
    <strong>{demand} support · {opposition} opposition ({total} total)</strong>
    <p class="helper-copy">
      Support share: {total === 0 ? 0 : Math.round((demand / total) * 100)}%
    </p>
  </div>
  <p class="helper-copy">
    Approval needs {required} of {plural(members, '{n} member', '{n} members')} to vote and at least 66%
    yes.
  </p>
  <textarea
    bind:value={form.demandNote}
    rows="4"
    aria-label={t('Support it answers')}
    placeholder={t('Covers the 3 who asked, weekly.')}></textarea>
{:else if step.id === 'schedule-mode' && isMeetupForm(form)}
  <MeetupSchedule bind:form />
{:else if step.id === 'schedule-mode'}
  <div class="plan-opts" role="radiogroup" aria-label={t('Schedule')}>
    <label
      ><input type="radio" bind:group={form.scheduleMode} value="date" /><b>{t('Single date')}</b
      ><small>{t('One day, with a start and finish time.')}</small></label
    >
    <label
      ><input type="radio" bind:group={form.scheduleMode} value="range" /><b>{t('Date range')}</b
      ><small>{t('Several days in a row, like a festival.')}</small></label
    >
  </div>
{:else if step.id === 'schedule-date'}
  <input type="date" bind:value={form.scheduledDate} aria-label={t('Date')} />
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
  <PlanStagesEditor bind:stages={form.stages} />
{/if}

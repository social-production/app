<script lang="ts">
  import {
    SOFTWARE_LICENSE_BODY,
    SOFTWARE_LICENSE_LABEL,
    SOFTWARE_LICENSE_TITLE,
    SOFTWARE_LICENSE_WHY,
  } from '#lib/domain/policy.ts';
  import { zoneOf } from '#lib/domain/activityTime.ts';
  import { scheduleProblem } from '#lib/domain/eventSchedule.ts';
  import {
    sameAsProduction,
    withPlanPlace,
    type PlanForm,
    type PlanStep,
    type ProductionLocation,
  } from '#lib/domain/planForm.ts';
  import type { PlaceSearch } from '#lib/domain/places.ts';
  import LocationPicker from '#lib/components/location/LocationPicker.svelte';
  import { REQUEST_MODES, type ProjectSubtype } from '#lib/domain/projects.ts';

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
  const zone = Intl.DateTimeFormat().resolvedOptions().timeZone;
</script>

{#if step.id === 'plan-overview'}
  <div class="overview-stack">
    <label>
      <span class="field-inline-label">Title</span>
      <input bind:value={form.title} maxlength="120" placeholder="Plan title" />
    </label>
    <label>
      <span class="field-inline-label">Description</span>
      <textarea bind:value={form.description} rows="5" placeholder="Describe the plan"></textarea>
    </label>
  </div>
{:else if step.id === 'subtype'}
  <select bind:value={form.subtype} aria-label="Plan type">
    {#each subtypes as option (option.value)}
      <option value={option.value}>{option.label}</option>
    {/each}
  </select>
{:else if step.id === 'repository'}
  <aside class="license-notice" aria-label="Software license guidance">
    <strong>License: {SOFTWARE_LICENSE_LABEL}</strong>
    <p>{SOFTWARE_LICENSE_TITLE}</p>
    <p>{SOFTWARE_LICENSE_BODY}</p>
    <ul>
      {#each SOFTWARE_LICENSE_WHY as reason (reason)}<li>{reason}</li>{/each}
    </ul>
  </aside>
  <input
    bind:value={form.repositoryUrl}
    aria-label="Repository URL"
    placeholder="https://github.com/org/repo"
  />
{:else if step.id === 'location'}
  <div class="location-step">
    <LocationPicker
      value={form.place}
      modes={['physical', 'online']}
      portaled
      elevated
      onChange={(value) => (form = withPlanPlace(form, value))}
      search={places.search}
      failure={places.failure}
    />
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
        <span>Same as production plan location ({productionLocation.label})</span>
      </label>
    {/if}
    {#if !form.sameAsProduction}
      <LocationPicker
        value={form.place}
        modes={['physical', 'online']}
        portaled
        elevated
        onChange={(value) => (form = withPlanPlace(form, value))}
        search={places.search}
        failure={places.failure}
      />
    {/if}
    <label class="recipients-field">
      <span>Recipients</span>
      <textarea
        bind:value={form.recipients}
        rows="2"
        maxlength="500"
        placeholder="Who receives what this project makes?"></textarea>
    </label>
  </div>
{:else if step.id === 'requests'}
  <select bind:value={form.requests} aria-label="Requests">
    <option value="">No requests for now</option>
    {#each REQUEST_MODES as option (option.value)}
      <option value={option.value}>{option.label}</option>
    {/each}
  </select>
  {#if form.requests}
    <p class="helper-copy">{REQUEST_MODES.find((m) => m.value === form.requests)?.summary}</p>
  {/if}
{:else if step.id === 'demand-note'}
  <div class="signal-card" aria-label="Current support signals">
    <span class="signal-label">Current support signals</span>
    <strong>{demand} support · {opposition} opposition ({total} total)</strong>
    <p class="helper-copy">
      Support share: {total === 0 ? 0 : Math.round((demand / total) * 100)}%
    </p>
  </div>
  <p class="helper-copy">
    Approval needs {required} of {members}
    {members === 1 ? 'member' : 'members'} to vote and at least 66% yes.
  </p>
  <textarea
    bind:value={form.demandNote}
    rows="5"
    placeholder="Explain how this plan responds to support"></textarea>
{:else if step.id === 'schedule-mode'}
  <select bind:value={form.scheduleMode} aria-label="Schedule">
    <option value="date">Single date</option>
    <option value="range">Date range</option>
  </select>
{:else if step.id === 'schedule-date'}
  <input type="date" bind:value={form.scheduledDate} aria-label="Date" />
{:else if step.id === 'schedule-range'}
  <div class="field-grid">
    <label>
      <span>Start date</span>
      <input type="date" bind:value={form.rangeStartDate} />
    </label>
    <label>
      <span>End date</span>
      <input type="date" bind:value={form.rangeEndDate} />
    </label>
  </div>
{:else if step.id === 'schedule-time'}
  <div class="field-grid">
    <label>
      <span>Start time</span>
      <input type="time" bind:value={form.startTime} />
    </label>
    <label>
      <span>Finish time</span>
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

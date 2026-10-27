<script lang="ts">
  import {
    SOFTWARE_LICENSE_BODY,
    SOFTWARE_LICENSE_LABEL,
    SOFTWARE_LICENSE_TITLE,
    SOFTWARE_LICENSE_WHY,
  } from '#lib/domain/policy.ts';
  import { zoneOf } from '#lib/domain/activityTime.ts';
  import { scheduleProblem } from '#lib/domain/eventSchedule.ts';
  import type { PlanForm, PlanStep } from '#lib/domain/planForm.ts';
  import type { ProjectSubtype } from '#lib/domain/projects.ts';

  import PlanStagesEditor from './PlanStagesEditor.svelte';

  /**
   * The fields of one plan wizard step (web's PlanCreationWizard body). Locations are plain
   * labels, like the charter's; web's place search arrives with the location work.
   */
  interface Props {
    step: PlanStep;
    form: PlanForm;
    subtypes: { value: ProjectSubtype; label: string }[];
    /** The approved production plan's location, for "Same as production plan location". */
    productionLocation: string | null;
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
    demand,
    opposition,
    required,
    members,
  }: Props = $props();

  const total = $derived(demand + opposition);
  const zone = Intl.DateTimeFormat().resolvedOptions().timeZone;

  function sameAsProduction(checked: boolean): void {
    form.sameAsProduction = checked;
    form.location = checked ? (productionLocation ?? '') : '';
  }
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
    <input
      bind:value={form.location}
      aria-label="Location"
      placeholder="Place, address or Online"
    />
  </div>
{:else if step.id === 'distribution-location'}
  <div class="settings-stack location-step">
    {#if productionLocation}
      <label class="checkbox-row">
        <input
          checked={form.sameAsProduction}
          type="checkbox"
          onchange={(e) => sameAsProduction(e.currentTarget.checked)}
        />
        <span>Same as production plan location ({productionLocation})</span>
      </label>
    {/if}
    {#if !form.sameAsProduction}
      <input
        bind:value={form.location}
        aria-label="Distribution location"
        placeholder="Place, address or Online"
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

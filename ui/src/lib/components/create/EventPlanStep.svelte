<script lang="ts">
  import RequiredLabel from './RequiredLabel.svelte';
  import { zoneOf } from '#lib/domain/activityTime.ts';
  import type { PlanForm } from '#lib/domain/planForm.ts';

  /**
   * web's create-event Plan step for organizer-controlled private events: the plan, its
   * schedule and its stages, set up front so the event starts in Activity (ADR 0061).
   */
  interface Props {
    plan: PlanForm;
    problems: string[];
  }

  let { plan = $bindable(), problems }: Props = $props();

  const zone = Intl.DateTimeFormat().resolvedOptions().timeZone;

  function addStage(): void {
    plan.stages = [...plan.stages, { title: '', details: '', startDate: '', endDate: '' }];
  }

  function removeStage(index: number): void {
    if (plan.stages.length > 1) plan.stages = plan.stages.filter((_, i) => i !== index);
  }
</script>

<div class="form-stack plan-step">
  <p class="helper-text">
    Organizer-controlled private events skip signals, values, and proposal votes. Set the plan now
    so the event can start in Activity. Members can still join and sign up for roles once activities
    exist.
  </p>
  <label>
    <RequiredLabel>Plan title</RequiredLabel>
    <input bind:value={plan.title} aria-required="true" />
  </label>
  <label>
    <RequiredLabel>Plan description</RequiredLabel>
    <textarea bind:value={plan.description} rows="3" aria-required="true"></textarea>
  </label>
  <fieldset class="schedule-fieldset">
    <legend>Schedule</legend>
    <div class="schedule-mode-row" role="radiogroup" aria-label="Schedule mode">
      <label class="schedule-mode-option" class:active={plan.scheduleMode === 'date'}>
        <input type="radio" bind:group={plan.scheduleMode} value="date" />
        Single date
      </label>
      <label class="schedule-mode-option" class:active={plan.scheduleMode === 'range'}>
        <input type="radio" bind:group={plan.scheduleMode} value="range" />
        Date range
      </label>
    </div>
    <div class="schedule-fields" class:is-range={plan.scheduleMode === 'range'}>
      {#if plan.scheduleMode === 'date'}
        <label>
          <RequiredLabel>Date</RequiredLabel>
          <input type="date" bind:value={plan.scheduledDate} />
        </label>
      {:else}
        <label>
          <RequiredLabel>Start date</RequiredLabel>
          <input type="date" bind:value={plan.rangeStartDate} />
        </label>
        <label>
          <RequiredLabel>End date</RequiredLabel>
          <input type="date" bind:value={plan.rangeEndDate} />
        </label>
      {/if}
    </div>
    <div class="schedule-fields is-range">
      <label>
        <RequiredLabel>Start time</RequiredLabel>
        <input type="time" bind:value={plan.startTime} />
      </label>
      <label>
        <RequiredLabel>Finish time</RequiredLabel>
        <input type="time" bind:value={plan.finishTime} />
      </label>
    </div>
    <!-- web's TimeZoneNote, without its Settings link: the app has no display time zone yet. -->
    <p class="helper-text">
      Times are in {zoneOf(new Date()) ? `${zoneOf(new Date())} (${zone})` : zone}.
    </p>
  </fieldset>
  <div class="stages">
    <div class="stages-header">
      <div>
        <strong>Plan stages</strong>
        <p class="stages-helper">Break the plan into ordered stages people can follow.</p>
      </div>
      <button type="button" class="secondary-button" onclick={addStage}>Add stage</button>
    </div>
    {#each plan.stages as stage, index (index)}
      <div class="stage-card">
        <div class="stage-card-header">
          <strong>Stage {index + 1}</strong>
          {#if plan.stages.length > 1}
            <button type="button" class="remove-stage" onclick={() => removeStage(index)}>
              Remove
            </button>
          {/if}
        </div>
        <label>
          <RequiredLabel>Title</RequiredLabel>
          <input bind:value={stage.title} />
        </label>
        <label>
          <RequiredLabel>Details</RequiredLabel>
          <textarea bind:value={stage.details} rows="2"></textarea>
        </label>
      </div>
    {/each}
  </div>
  {#if problems.length > 0}
    <ul class="validation-list">
      {#each problems as message (message)}
        <li>{message}</li>
      {/each}
    </ul>
  {/if}
</div>

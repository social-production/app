<script lang="ts">
  import ActivityAgenda from '#lib/components/activities/ActivityAgenda.svelte';
  import ActivityCalendar from '#lib/components/activities/ActivityCalendar.svelte';
  import ActivityHistory from '#lib/components/activities/ActivityHistory.svelte';
  import ActivityWizard from '#lib/components/activities/ActivityWizard.svelte';
  import { activityQuickPicks } from '#lib/domain/activityForm.ts';
  import { dayWindow, planOpen, scheduleDays } from '#lib/domain/eventSchedule.ts';
  import { placeSearch } from '#lib/services/locations.ts';
  import { connection } from '#lib/stores/connection.svelte.ts';

  import { page } from '$app/state';

  import { ActivitiesState, stageOptions } from './activitiesState.svelte.ts';
  import type { DetailState } from './detailState.svelte.ts';
  import ServicePhase from './requests/ServicePhase.svelte';
  import SoftwareDock from './software/SoftwareDock.svelte';
  import SoftwarePanel from './software/SoftwarePanel.svelte';
  import { SoftwareState } from './software/softwareState.svelte.ts';
  import SoftwareWizard from './software/SoftwareWizard.svelte';

  import '#lib/components/styles/activities.css';

  /**
   * web's Activity phase (ProductiveLifecyclePhaseFive → ActivitySchedulingPanel): the calendar
   * with Add activity, the composer, upcoming activities as agenda rows, and Past activity
   * (check-ins and ratings). An event's (EventActivityPhase) marks its plan's days, takes
   * activities only on them and only until the plan ends, from organizers when they run it.
   * A personal service has its own (ServicePhase, ADR 0077).
   */
  let { detail, current }: { detail: DetailState; current: boolean } = $props();

  const me = $derived(connection.current.status === 'connected' ? connection.current.agent : '');
  // svelte-ignore state_referenced_locally
  const state = new ActivitiesState(detail.id, () => me);
  const project = $derived(detail.project);
  const personal = $derived(project?.charter?.mode === 'personal-service');
  const schedule = $derived(
    project?.charter?.mode === 'event'
      ? (detail.plans?.plans.find(
          (p) => p.plan.kind === 'Event' && p.leader === 'Leading' && p.approved,
        )?.plan.schedule ?? null)
      : undefined,
  );
  const planDays = $derived(schedule ? scheduleDays(schedule) : []);
  // web-backend: organizer-controlled events take activities from organizers only, and an
  // event only while its leading plan hasn't ended (`_can_propose_event_activity`).
  const mayAdd = $derived(
    project?.governance === 'organizer_controlled' ? !!project.amOrganizer : !!project?.amMember,
  );
  const canCreate = $derived(
    current &&
      mayAdd &&
      (!personal || project?.creator === me) &&
      (schedule === undefined || (!!schedule && planOpen(schedule))),
  );
  const composerDay = $derived(state.selectedDay || planDays[0] || '');
  const planWindow = $derived(schedule ? dayWindow(schedule, composerDay) : null);
  const stages = $derived(stageOptions(detail.plans, project?.charter?.mode ?? ''));
  const quickPicks = $derived(
    activityQuickPicks(
      project?.charter?.mode ?? '',
      { label: project?.charter?.location_label ?? '', place: project?.charter?.place },
      detail.plans?.plans ?? [],
    ),
  );

  // A software project's pull requests (ADR 0086), as web shows them above the calendar.
  const software = $derived(
    project?.charter?.subtype === 'software' &&
      ['productive', 'collective-service'].includes(project.charter.mode),
  );
  // svelte-ignore state_referenced_locally
  const sw = new SoftwareState(detail.id);
  $effect(() => {
    if (software) void sw.load();
  });

  $effect(() => {
    state.highlighted = page.url.searchParams.get('activity');
    void state.load();
  });

  function focus(id: string): void {
    state.highlighted = id;
    document.getElementById(`activity-card-${id}`)?.scrollIntoView({ block: 'center' });
  }
</script>

{#if personal}
  <ServicePhase {detail} />
{:else}
  <section id="participation-activities" class="phase-surface activity-ui">
    {#if software}
      {#if sw.governance}
        <SoftwarePanel {sw} />
        <SoftwareWizard {sw} />
      {:else}
        <div class="software-governance-placeholder">
          <h3>Software governance</h3>
          <p>
            Pull request tools appear here once a leading software plan is approved for this
            project.
          </p>
        </div>
      {/if}
    {/if}
    <section class="scheduling-panel">
      <ActivityCalendar
        activities={state.activities}
        selectedDay={state.selectedDay}
        selectedActivity={state.highlighted ?? ''}
        {planDays}
        {canCreate}
        createActive={state.composing}
        onDay={(iso) => (state.selectedDay = iso)}
        onActivity={focus}
        onCreate={() =>
          state.composing ? (state.composing = false) : state.openComposer(composerDay, planWindow)}
        extra={software && sw.governance?.canPropose ? dock : undefined}
      />
      <section class="card-rail-section live-section">
        {#if state.error && !state.composing}
          <p class="validation-copy" role="alert">{state.error}</p>
        {/if}
        {#if !state.loaded}
          <div class="empty-card">Loading activities…</div>
        {:else if state.live.length === 0}
          <div class="empty-card">
            {planDays.length > 0
              ? 'No activity scheduled yet. Click a marked plan day to add the first activity.'
              : 'No activities scheduled yet.'}
          </div>
        {:else}
          <ActivityAgenda
            activities={state.live}
            names={state.names}
            {me}
            planStageOf={(id) => stages.find((s) => s.id === id)?.label ?? null}
            highlighted={state.highlighted}
            readOnly={!me}
            canSuggest={!!project?.amMember}
            onCommit={state.commit}
            onSuggest={state.suggest}
            onDecline={state.decline}
            searchPeople={state.searchPeople}
          />
        {/if}
      </section>
      {#if state.past.length > 0}
        <details class="history-section">
          <summary class="history-summary">
            <span>Past activity</span>
            <svg
              class="history-chevron"
              aria-hidden="true"
              viewBox="0 0 24 24"
              width="18"
              height="18"
            >
              <path
                d="M6 9l6 6 6-6"
                fill="none"
                stroke="currentColor"
                stroke-width="2.2"
                stroke-linecap="round"
                stroke-linejoin="round"
              />
            </svg>
          </summary>
          <ActivityHistory
            items={state.past}
            names={state.names}
            {me}
            planStageOf={(id) => stages.find((s) => s.id === id)?.label ?? null}
            onCheckIn={state.checkIn}
            onRate={state.rate}
          />
        </details>
      {/if}
    </section>
    {#if canCreate}
      <ActivityWizard
        open={state.composing}
        bind:form={state.form}
        planStages={stages}
        {planWindow}
        busy={state.busy}
        error={state.error}
        onSubmit={() => void state.submit()}
        onCancel={() => (state.composing = false)}
        searchPeople={state.searchPeople}
        places={placeSearch}
        {quickPicks}
      />
    {/if}
  </section>
{/if}

{#snippet dock()}
  <SoftwareDock {sw} />
{/snippet}

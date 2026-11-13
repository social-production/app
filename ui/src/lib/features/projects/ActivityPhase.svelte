<script lang="ts">
  import ActivityAgenda from '#lib/components/activities/ActivityAgenda.svelte';
  import ActivityCalendar from '#lib/components/activities/ActivityCalendar.svelte';
  import ActivityHistory from '#lib/components/activities/ActivityHistory.svelte';
  import ActivityWizard from '#lib/components/activities/ActivityWizard.svelte';
  import DecisionHistoryCard from '#lib/components/governance/DecisionHistoryCard.svelte';
  import SegmentedChoice from '#lib/components/SegmentedChoice.svelte';
  import type { Activity } from '#lib/domain/activities.ts';
  import { activityQuickPicks } from '#lib/domain/activityForm.ts';
  import { dayWindow, planOpen, scheduleDays } from '#lib/domain/eventSchedule.ts';
  import { isSoftwareProject } from '#lib/domain/software.ts';
  import { placeSearch } from '#lib/services/locations.ts';
  import { connection } from '#lib/stores/connection.svelte.ts';

  import { page } from '$app/state';

  import { ActivitiesState, stageOptions } from './activitiesState.svelte.ts';
  import type { DetailState } from './detailState.svelte.ts';
  import CollectiveRequests from './requests/CollectiveRequests.svelte';
  import ServicePhase from './requests/ServicePhase.svelte';
  import SoftwareDock from './software/SoftwareDock.svelte';
  import SoftwarePanel from './software/SoftwarePanel.svelte';
  import { openSoftwareLink } from './software/softwareLink.ts';
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
  // A collective service's requests (ADR 0089): planned requests are accepted once written.
  const collectiveService = $derived(project?.charter?.mode === 'collective-service');
  const collective: { accept?: (request: string) => void } = {};

  async function submitActivity(): Promise<void> {
    const planned = state.planning;
    if ((await state.submit()) && planned) collective.accept?.(planned.request);
  }
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

  // A software project's pull requests (ADR 0086), as web shows them above the calendar;
  // the page reads them (DetailState), and a rail link opens a pull request's wizard.
  const software = $derived(isSoftwareProject(project?.charter));
  const sw = $derived(detail.software);
  $effect(() => openSoftwareLink(sw, page.url.searchParams));
  // web's Past activity: ended activities and software decisions, newest first.
  const pastTimeline = $derived(
    [
      ...state.past.map((a) => ({ id: a.id, at: a.startsAt, activity: a, entry: null })),
      ...sw.history.map((e) => ({ id: `gov-${e.id}`, at: e.createdAt, activity: null, entry: e })),
    ].sort((a, b) => b.at - a.at),
  );

  // Upcoming and Past as tabs (ADR 0102), in place of web's fold at the bottom.
  const hasPast = $derived(state.past.length + sw.history.length > 0);
  const views = [
    { value: 'upcoming' as const, label: 'Upcoming' },
    { value: 'past' as const, label: 'Past activity' },
  ];

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
    <section class="scheduling-panel" class:showing-past={hasPast && state.showing === 'past'}>
      {#if hasPast}
        <SegmentedChoice
          value={state.showing}
          ariaLabel="Activities shown"
          options={views}
          onChange={(v) => (state.showing = v)}
        />
      {/if}
      <ActivityCalendar
        week
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
      {#if state.showing === 'upcoming' || !hasPast}
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
      {:else}
        <section class="history-section past-tab" aria-label="Past activity">
          {#if sw.history.length === 0}
            {@render past(state.past)}
          {:else}
            <!-- web's unified history: a software project's decisions among its activities. -->
            <div class="unified-history">
              {#each pastTimeline as row (row.id)}
                {#if row.activity}
                  {@render past([row.activity])}
                {:else if row.entry && project?.charter}
                  <DecisionHistoryCard
                    entry={row.entry}
                    names={{ ...detail.names, ...sw.names }}
                    charter={project.charter}
                    stageOf={() => project?.stage ?? ''}
                    phaseLabel={detail.phaseLabel}
                    onVote={(p, c) => detail.vote(p, c)}
                  />
                {/if}
              {/each}
            </div>
          {/if}
        </section>
      {/if}
    </section>
    {#if collectiveService}
      <CollectiveRequests
        register={(accept) => (collective.accept = accept)}
        {detail}
        activities={state}
        {me}
        canPlan={canCreate}
      />
    {/if}
    {#if canCreate}
      <ActivityWizard
        open={state.composing}
        bind:form={state.form}
        planStages={stages}
        {planWindow}
        busy={state.busy}
        error={state.error}
        onSubmit={() => void submitActivity()}
        onCancel={() => (state.composing = false)}
        searchPeople={state.searchPeople}
        places={placeSearch}
        {quickPicks}
      />
    {/if}
  </section>
{/if}

{#snippet past(items: Activity[])}
  <ActivityHistory
    {items}
    names={state.names}
    {me}
    planStageOf={(id) => stages.find((s) => s.id === id)?.label ?? null}
    onCheckIn={state.checkIn}
    onRate={state.rate}
  />
{/snippet}

{#snippet dock()}
  <SoftwareDock {sw} />
{/snippet}

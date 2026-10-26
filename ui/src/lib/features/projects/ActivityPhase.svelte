<script lang="ts">
  import ActivityAgenda from '#lib/components/activities/ActivityAgenda.svelte';
  import ActivityCalendar from '#lib/components/activities/ActivityCalendar.svelte';
  import ActivityHistory from '#lib/components/activities/ActivityHistory.svelte';
  import ActivityWizard from '#lib/components/activities/ActivityWizard.svelte';
  import { connection } from '#lib/stores/connection.svelte.ts';

  import { page } from '$app/state';

  import { ActivitiesState, stageOptions } from './activitiesState.svelte.ts';
  import type { DetailState } from './detailState.svelte.ts';

  import '#lib/components/styles/activities.css';

  /**
   * web's Activity phase (ProductiveLifecyclePhaseFive → ActivitySchedulingPanel): the calendar
   * with Add activity, the composer, upcoming activities as agenda rows, and Past activity
   * (check-ins and ratings).
   */
  let { detail, current }: { detail: DetailState; current: boolean } = $props();

  const me = $derived(connection.current.status === 'connected' ? connection.current.agent : '');
  // svelte-ignore state_referenced_locally
  const state = new ActivitiesState(detail.id, () => me);
  const project = $derived(detail.project);
  const personal = $derived(project?.charter?.mode === 'personal-service');
  const canCreate = $derived(
    current && !!project?.amMember && (!personal || project.creator === me),
  );
  const stages = $derived(stageOptions(detail.plans, project?.charter?.mode ?? ''));

  $effect(() => {
    state.highlighted = page.url.searchParams.get('activity');
    void state.load();
  });

  function focus(id: string): void {
    state.highlighted = id;
    document.getElementById(`activity-card-${id}`)?.scrollIntoView({ block: 'center' });
  }
</script>

<section id="participation-activities" class="phase-surface activity-ui">
  <section class="scheduling-panel">
    <ActivityCalendar
      activities={state.activities}
      selectedDay={state.selectedDay}
      selectedActivity={state.highlighted ?? ''}
      {canCreate}
      createActive={state.composing}
      onDay={(iso) => (state.selectedDay = iso)}
      onActivity={focus}
      onCreate={() => (state.composing ? (state.composing = false) : state.openComposer())}
    />
    <section class="card-rail-section live-section">
      {#if state.error && !state.composing}
        <p class="validation-copy" role="alert">{state.error}</p>
      {/if}
      {#if !state.loaded}
        <div class="empty-card">Loading activities…</div>
      {:else if state.live.length === 0}
        <div class="empty-card">No activities scheduled yet.</div>
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
      busy={state.busy}
      error={state.error}
      onSubmit={() => void state.submit()}
      onCancel={() => (state.composing = false)}
      searchPeople={state.searchPeople}
    />
  {/if}
</section>

<script lang="ts">
  import { t } from '#lib/i18n/index.ts';
  import PlanWizardShell from '#lib/components/plans/PlanWizardShell.svelte';
  import {
    activitySteps,
    localNow,
    stepReady,
    type ActivityForm,
    type ActivityStepType,
  } from '#lib/domain/activityForm.ts';
  import { dayOf, nextWeekday, onDay, stampOf, usualOf } from '#lib/domain/activityWhen.ts';
  import { dayWindow, scheduleDays, type EventSchedule } from '#lib/domain/eventSchedule.ts';
  import type { PlaceQuickPick, PlaceSearch } from '#lib/domain/places.ts';

  import ActivityReview from './ActivityReview.svelte';
  import ActivityWhenStep from './ActivityWhenStep.svelte';
  import ActivityWhereStep from './ActivityWhereStep.svelte';
  import RolesEditor from './RolesEditor.svelte';

  import '#lib/components/styles/activities.css';
  import '#lib/components/styles/ux/activitywizard.css';

  /**
   * Creating an activity in four steps (Activity creation mockup, owner 2026-11-25): what and
   * when (a month calendar and time wheels), where (the plan's place first) and which plan
   * stage, who's needed, then check and create. A meetup's or event's is a "session".
   */
  interface Props {
    open: boolean;
    form: ActivityForm;
    planStages: { id: string; label: string; group: string }[];
    /** An event's or meetup's leading plan schedule: its days, its usual weekday. */
    schedule?: EventSchedule | null;
    /** A meetup or event: "New session". */
    session?: boolean;
    busy: boolean;
    error: string;
    onSubmit: () => void;
    onCancel: () => void;
    searchPeople: (q: string) => Promise<{ agent: string; name: string }[]>;
    places: PlaceSearch;
    quickPicks: PlaceQuickPick[];
  }

  let {
    open,
    form = $bindable(),
    planStages,
    schedule = null,
    session = false,
    busy,
    error,
    onSubmit,
    onCancel,
    searchPeople,
    places,
    quickPicks,
  }: Props = $props();

  let index = $state(0);
  const steps = activitySteps();
  const step = $derived(steps[index] ?? steps[0]);
  const today = $derived(dayOf(localNow()));
  const allowedDays = $derived(
    schedule && (schedule.mode === 'date' || schedule.mode === 'range')
      ? scheduleDays(schedule)
      : [],
  );
  const minDay = $derived(allowedDays[0] && allowedDays[0] > today ? allowedDays[0] : today);
  const usual = $derived(usualOf(schedule));
  // An event's plan day bounds the times (web's `scheduleBounds`).
  const window = $derived(schedule ? dayWindow(schedule, dayOf(form.scheduledAt)) : null);
  const outside = $derived(!!window && (form.scheduledAt < window.min || form.endsAt > window.max));
  const stageLabel = $derived(planStages.find((s) => s.id === form.planStage)?.label ?? null);
  const ready = $derived(
    !!step && stepReady(step.type, form, planStages.length > 0) && !(index === 0 && outside),
  );

  // A meetup opens on its next usual day and times.
  let seeded = false;
  $effect(() => {
    if (!open) {
      seeded = false;
      return;
    }
    if (seeded || !usual) return;
    seeded = true;
    const day = nextWeekday(usual.weekday, today);
    const times = onDay(form, day);
    form = {
      ...form,
      scheduledAt: usual.start ? stampOf(day, usual.start) : times.scheduledAt,
      endsAt: usual.finish ? stampOf(day, usual.finish) : times.endsAt,
    };
  });

  function close(): void {
    index = 0;
    onCancel();
  }

  function next(): void {
    if (step?.type === 'review') onSubmit();
    else index += 1;
  }

  const goTo = (type: ActivityStepType): void => {
    index = Math.max(
      0,
      steps.findIndex((s) => s.type === type),
    );
  };
</script>

<PlanWizardShell
  {open}
  title={session ? t('New session') : t('New activity')}
  stepIndex={index}
  stepCount={steps.length}
  nextLabel={step?.type === 'review'
    ? busy
      ? t('Creating…')
      : session
        ? t('Create session')
        : t('Create activity')
    : t('Next')}
  canGoBack={index > 0}
  canGoNext={!busy && ready}
  onClose={close}
  onDismiss={onCancel}
  onBack={() => (index = Math.max(0, index - 1))}
  onNext={next}
>
  {#if step}
    <div class="activity-wizard activity-wizard-v2">
      <h2>{step.question}</h2>
      {#if step.type === 'when'}
        <ActivityWhenStep
          {form}
          {minDay}
          {allowedDays}
          usualWeekday={usual?.weekday ?? null}
          onChange={(next) => (form = next)}
        />
        {#if outside && window}
          <p class="validation-copy">{t("Keep it within the plan's times that day.")}</p>
        {/if}
      {:else if step.type === 'where'}
        <ActivityWhereStep
          {form}
          {quickPicks}
          {places}
          stages={planStages}
          onChange={(next) => (form = next)}
        />
      {:else if step.type === 'who'}
        {#if session}<p class="helper-copy">
            {t('Leave it as Players for a session anyone can join.')}
          </p>{/if}
        <RolesEditor bind:roles={form.roles} {searchPeople} />
      {:else}
        <ActivityReview
          {form}
          {stageLabel}
          {error}
          onEdit={goTo}
          onNote={(note) => (form = { ...form, note })}
        />
      {/if}
    </div>
  {/if}
</PlanWizardShell>

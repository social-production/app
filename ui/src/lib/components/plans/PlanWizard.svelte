<script lang="ts">
  import { t } from '#lib/i18n/index.ts';
  import type { PlaceSearch } from '#lib/domain/places.ts';
  import type { ProductionLocation } from '#lib/domain/planForm.ts';
  import {
    emptyPlanForm,
    isMeetupForm,
    planFromForm,
    planSteps,
    stepReady,
    type PlanStepType,
  } from '#lib/domain/planForm.ts';
  import { PLAN_KIND_NAMES, subtypeOptions, type Plan, type PlanKind } from '#lib/domain/plans.ts';
  import { isProjectMode, type CharterMode, type ProjectSubtype } from '#lib/domain/projects.ts';

  import CreateSheet from '#lib/components/create/CreateSheet.svelte';
  import PolicyNotice from '#lib/components/create/PolicyNotice.svelte';

  import PlanReview from './PlanReview.svelte';
  import PlanWizardStep from './PlanWizardStep.svelte';

  import '#lib/components/styles/ux/planstages.css';

  /**
   * web's PlanCreationWizard for project plans (overview, subtype, repository or location,
   * support response, stages, review) and event plans (overview, schedule, place, support
   * response, stages, review), in the create flows' sheet (Edit, Plan and Meetup mockups §B,
   * owner 2026-11-24): one question per step, the direct-use rule as a chip, Back | Continue in
   * the dock. Closing with something typed asks first, then starts over.
   */
  interface Props {
    open: boolean;
    kind: PlanKind;
    /** An event plan with a usual day or none: makes a meetup (ADR 0119). */
    meetup?: boolean;
    mode: CharterMode;
    subtype: ProjectSubtype;
    productionLocation: ProductionLocation | null;
    places: PlaceSearch;
    demand: number;
    opposition: number;
    required: number;
    members: number;
    pending: boolean;
    message: string | undefined;
    /** Resolves true once filed. */
    onSubmit: (plan: Plan) => Promise<boolean>;
    onDismiss: () => void;
  }

  let {
    open,
    kind,
    meetup = false,
    mode,
    subtype,
    productionLocation,
    places,
    demand,
    opposition,
    required,
    members,
    pending,
    message,
    onSubmit,
    onDismiss,
  }: Props = $props();

  // The wizard opens fresh per kind; the project's subtype seeds the draft once.
  // svelte-ignore state_referenced_locally
  let form = $state(emptyPlanForm(subtype));
  let index = $state(0);

  const steps = $derived(planSteps(kind, form));
  const step = $derived(steps[Math.min(index, steps.length - 1)]);
  const reviewing = $derived(step?.id === 'review');
  const name = $derived(meetup && kind === 'Event' ? 'meetup plan' : PLAN_KIND_NAMES[kind]);

  // A meetup plan starts on a usual day; an event plan on a date.
  $effect(() => {
    const undated = meetup && kind === 'Event';
    if (undated !== isMeetupForm(form)) form.scheduleMode = undated ? 'weekly' : 'date';
  });
  // "Distribution plan" at the top of the sheet.
  const title = $derived.by(() => {
    const words = t(name);
    return words.charAt(0).toUpperCase() + words.slice(1);
  });

  function reset(): void {
    form = emptyPlanForm(subtype);
    index = 0;
  }

  function close(): void {
    reset();
    onDismiss();
  }

  function goTo(id: PlanStepType): void {
    index = Math.max(
      steps.findIndex((s) => s.id === id),
      0,
    );
  }

  async function submit(): Promise<void> {
    if (await onSubmit(planFromForm(kind, form))) reset();
  }
</script>

{#if open}
  <CreateSheet
    {title}
    description=""
    steps={steps.map((s) => ({ id: s.id, title: s.question, question: s.question }))}
    stepIndex={Math.min(index, steps.length - 1)}
    missing={step && stepReady(step.id, form) ? [] : [t('the required fields')]}
    submitLabel={t('Propose plan')}
    submittingLabel={t('Working...')}
    submitting={pending}
    onStep={(i) => (index = i)}
    onSubmit={() => void submit()}
    onClose={close}
  >
    <div class="form-stack plan-wizard">
      {#if message && reviewing}
        <div class="warning-card" role="alert">
          <strong>{t('Plan could not be submitted')}</strong>
          <ul class="warning-list"><li>{message}</li></ul>
        </div>
      {/if}
      {#if step}
        {#if step.helper}<p class="helper-text">{step.helper}</p>{/if}
        {#if reviewing}
          <PlanReview {form} {steps} onEdit={goTo} />
        {:else}
          <PlanWizardStep
            {step}
            bind:form
            subtypes={isProjectMode(mode) ? subtypeOptions(mode) : []}
            {productionLocation}
            {places}
            {demand}
            {opposition}
            {required}
            {members}
            dated={kind === 'Event'}
          />
        {/if}
      {/if}
      {#if index === 0}<PolicyNotice />{/if}
    </div>
  </CreateSheet>
{/if}

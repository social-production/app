<script lang="ts">
  import { DIRECT_USE_CORE, DIRECT_USE_PLAN } from '#lib/domain/policy.ts';
  import {
    emptyPlanForm,
    planFromForm,
    planSteps,
    PLAN_KIND_NAMES,
    stepReady,
    subtypeOptions,
    type Plan,
    type PlanKind,
    type PlanStepType,
  } from '#lib/domain/plans.ts';
  import type { ProjectMode, ProjectSubtype } from '#lib/domain/projects.ts';

  import PlanReview from './PlanReview.svelte';
  import PlanWizardShell from './PlanWizardShell.svelte';
  import PlanWizardStep from './PlanWizardStep.svelte';

  /**
   * web's PlanCreationWizard for project plans: overview, subtype, repository or location,
   * support response, stages, review. Dismissing keeps the draft; closing starts over.
   */
  interface Props {
    open: boolean;
    kind: PlanKind;
    mode: ProjectMode;
    subtype: ProjectSubtype;
    productionLocation: string | null;
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
    mode,
    subtype,
    productionLocation,
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

  const steps = $derived(planSteps(kind, form.subtype));
  const step = $derived(steps[Math.min(index, steps.length - 1)]);
  const reviewing = $derived(step?.id === 'review');
  const name = $derived(PLAN_KIND_NAMES[kind]);

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

  async function next(): Promise<void> {
    if (!reviewing) {
      index = Math.min(index + 1, steps.length - 1);
      return;
    }
    if (await onSubmit(planFromForm(kind, form))) reset();
  }
</script>

<PlanWizardShell
  {open}
  title={`Create ${name}`}
  stepIndex={Math.min(index, steps.length - 1)}
  stepCount={steps.length}
  nextLabel={reviewing ? (pending ? 'Working...' : `Submit ${name}`) : 'Next'}
  canGoBack={index > 0}
  canGoNext={!!step && !pending && stepReady(step.id, form)}
  onClose={close}
  {onDismiss}
  onBack={() => (index = Math.max(index - 1, 0))}
  onNext={() => void next()}
>
  <div class="plan-wizard">
    {#if index === 0}
      <div class="policy-notice-wrap">
        <aside class="policy-notice" aria-label="Direct-use policy">
          <strong>Direct use only — no commercial activity</strong>
          <p>{DIRECT_USE_PLAN}</p>
          <p class="policy-core">{DIRECT_USE_CORE}</p>
        </aside>
      </div>
    {/if}
    {#if message && reviewing}
      <div class="warning-card" role="alert">
        <strong>Plan could not be submitted</strong>
        <ul class="warning-list"><li>{message}</li></ul>
      </div>
    {/if}
    {#if step}
      <div class="question-block">
        <h2>{step.question}</h2>
        {#if step.helper}<p class="helper-copy">{step.helper}</p>{/if}
        {#if reviewing}
          <PlanReview {form} {steps} onEdit={goTo} />
        {:else}
          <PlanWizardStep
            {step}
            bind:form
            subtypes={subtypeOptions(mode)}
            {productionLocation}
            {demand}
            {opposition}
            {required}
            {members}
          />
        {/if}
      </div>
    {/if}
  </div>
</PlanWizardShell>

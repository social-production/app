<script lang="ts">
  import PlanCard from '#lib/components/plans/PlanCard.svelte';
  import PlanWizard from '#lib/components/plans/PlanWizard.svelte';
  import RoundPlusButton from '#lib/components/RoundPlusButton.svelte';
  import type { PlanKind, Plan } from '#lib/domain/plans.ts';

  import type { DetailState } from './detailState.svelte.ts';

  import '#lib/components/styles/plans.css';

  /**
   * web's ProjectLifecyclePlanPhaseContent: the plans of one plan phase, members' assessment,
   * and the wizard to add one while that phase is current.
   */
  interface Props {
    detail: DetailState;
    kind: PlanKind;
  }

  let { detail, kind }: Props = $props();

  let composing = $state(false);

  const project = $derived(detail.project);
  const board = $derived(detail.plans);
  const plans = $derived(board?.plans.filter((p) => p.plan.kind === kind) ?? []);
  const canSubmit = $derived(!!project?.amMember && board?.openKind === kind);
  const productionLocation = $derived(
    board?.plans.find((p) => p.plan.kind === 'Production' && p.leader === 'Leading')?.plan
      .location ?? null,
  );

  async function submit(plan: Plan): Promise<boolean> {
    const ok = await detail.submitPlan(plan);
    if (ok) composing = false;
    return ok;
  }
</script>

{#if project?.charter}
  <section class="phase-surface plan-phase">
    {#if canSubmit}
      <PlanWizard
        open={composing}
        {kind}
        mode={project.charter.mode}
        subtype={project.charter.subtype}
        {productionLocation}
        demand={project.demand}
        opposition={project.opposition}
        required={board?.required ?? 0}
        members={board?.members ?? project.members.length}
        pending={detail.saving}
        message={detail.sheetMessage}
        onSubmit={submit}
        onDismiss={() => (composing = false)}
      />
    {/if}

    <div id="participation-plans">
      {#if plans.length === 0}
        <div class="phase-empty">
          <strong>No plans yet</strong>
          <p>Add a plan when you are ready to propose how this should work.</p>
        </div>
      {:else}
        <div class="plan-stack">
          {#each plans as plan (plan.id)}
            <PlanCard
              {plan}
              authorName={detail.names[plan.author] ?? '…'}
              canVote={project.amMember}
              onRate={(criterion, rating) => void detail.ratePlan(plan.id, criterion, rating)}
              onVote={(choice) => void detail.votePlan(plan.id, choice)}
            />
          {/each}
        </div>
      {/if}
    </div>

    {#if canSubmit}
      <div class="plan-toolbar">
        <RoundPlusButton
          standout
          active={composing}
          label="Add plan"
          onclick={() => (composing = !composing)}
        />
      </div>
    {/if}
  </section>
{/if}

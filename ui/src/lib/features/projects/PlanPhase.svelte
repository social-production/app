<script lang="ts">
  import PlanCard from '#lib/components/plans/PlanCard.svelte';
  import PlanWizard from '#lib/components/plans/PlanWizard.svelte';
  import PhaseWorkToolbar from '#lib/components/governance/PhaseWorkToolbar.svelte';
  import RoundPlusButton from '#lib/components/RoundPlusButton.svelte';
  import TypeSelector from '#lib/components/create/TypeSelector.svelte';
  import { PLAN_MAKES, planMakesLine, type PlanKind, type Plan } from '#lib/domain/plans.ts';
  import { projectSubtype } from '#lib/domain/projects.ts';
  import { placeSearch } from '#lib/services/locations.ts';

  import type { DetailState } from './detailState.svelte.ts';

  import '#lib/components/styles/plans.css';

  /**
   * web's ProjectLifecyclePlanPhaseContent: the plans of one plan phase, members' assessment,
   * and the wizard to add one while that phase is current.
   */
  interface Props {
    detail: DetailState;
    kind: PlanKind;
    /** A plan whose sheet a vote link opens; its link is dropped when the sheet closes. */
    revealPlan?: string | null;
    onRevealDone?: (() => void) | undefined;
  }

  let { detail, kind, revealPlan = null, onRevealDone }: Props = $props();

  let composing = $state(false);

  const project = $derived(detail.project);
  const board = $derived(detail.plans);
  // A proposal's Plan phase takes every type-giving kind side by side (ADR 0059).
  const typeless = $derived(project?.charter?.mode === 'proposal');
  const kinds = $derived<PlanKind[]>(typeless ? PLAN_MAKES.map((m) => m.value) : [kind]);
  const plans = $derived(board?.plans.filter((p) => kinds.includes(p.plan.kind)) ?? []);
  const canSubmit = $derived(
    !!project?.amMember && !!board?.openKinds.some((open) => kinds.includes(open)),
  );
  /** The kind of the plan being written: the phase's own, or the one picked on a proposal. */
  let chosen = $state<PlanKind>('Production');
  let choosing = $state(false);
  const writing = $derived(typeless ? chosen : kind);
  const writingMode = $derived(
    typeless ? (PLAN_MAKES.find((m) => m.value === chosen)?.mode ?? 'productive') : null,
  );

  function add(): void {
    if (composing || choosing) {
      composing = false;
      choosing = false;
    } else if (typeless) {
      choosing = true;
    } else {
      composing = true;
    }
  }
  const production = $derived(
    board?.plans.find((p) => p.plan.kind === 'Production' && p.leader === 'Leading')?.plan,
  );
  const productionLocation = $derived(
    production?.location ? { label: production.location, place: production.place ?? null } : null,
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
        kind={writing}
        mode={writingMode ?? project.charter.mode}
        subtype={projectSubtype(project.charter.subtype)}
        {productionLocation}
        places={placeSearch}
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

    {#if choosing}
      <div class="plan-kind-choice">
        <TypeSelector
          label="What kind of thing should this become?"
          name="plan-kind"
          selected={chosen}
          options={PLAN_MAKES}
          onSelect={(value) => (chosen = value)}
        />
        <button
          class="plan-kind-continue"
          type="button"
          onclick={() => {
            choosing = false;
            composing = true;
          }}>Continue</button
        >
      </div>
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
              makes={typeless ? planMakesLine(plan.plan.kind) : null}
              canVote={project.amMember}
              reveal={revealPlan === plan.id}
              {onRevealDone}
              onRate={(criterion, rating) => void detail.ratePlan(plan.id, criterion, rating)}
              onVote={(choice) => void detail.votePlan(plan.id, choice)}
            />
          {/each}
        </div>
      {/if}
    </div>

    <PhaseWorkToolbar>
      {#if canSubmit}
        <RoundPlusButton standout active={composing || choosing} label="Add plan" onclick={add} />
      {/if}
    </PhaseWorkToolbar>
  </section>
{/if}

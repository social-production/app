<script lang="ts">
  import { t } from '#lib/i18n/index.ts';
  import { canVoteOn } from '#lib/domain/board.ts';
  import PlanCard from '#lib/components/plans/PlanCard.svelte';
  import PlanWizard from '#lib/components/plans/PlanWizard.svelte';
  import PhaseWorkToolbar from '#lib/components/governance/PhaseWorkToolbar.svelte';
  import RoundPlusButton from '#lib/components/RoundPlusButton.svelte';
  import ChoiceRows from '#lib/components/create/ChoiceRows.svelte';
  import OverlaySheet from '#lib/components/overlay/OverlaySheet.svelte';
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
  // A box (project), a hand (service), a calendar (event), a ball (meetup).
  const PLAN_ICONS: Record<string, string> = {
    Production: 'M12 3 4 7.5v9L12 21l8-4.5v-9L12 3ZM4 7.5l8 4.5 8-4.5M12 12v9',
    Organisation: 'M3 15h4l3 2h5a2 2 0 0 1 0 4H8M8 13c0-5 3-8 9-9-.6 5.6-3.8 8.6-9 9Z',
    Event: 'M5 5h14v15H5zM5 10h14M9 3v4M15 3v4',
    Meetup:
      'M12 20a8 8 0 1 0 0-16 8 8 0 0 0 0 16ZM4 12h16M12 4c-2.5 2.2-3.5 5-3.5 8s1 5.8 3.5 8M12 4c2.5 2.2 3.5 5 3.5 8s-1 5.8-3.5 8',
  };

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

    <!-- Creating Through Plans mockup: "Add a plan" is a small sheet asking the kind first. -->
    <OverlaySheet
      open={choosing}
      title={t('Add a plan')}
      labelledById="plan-kind-title"
      rootClass="setting-sheet plan-kind-sheet"
      onClose={() => (choosing = false)}
    >
      {#snippet subtitle()}
        <p class="setting-hint">{t('What kind of thing should this become?')}</p>
      {/snippet}
      <div class="plan-kind-choice">
        <ChoiceRows
          value={chosen}
          ariaLabel={t('Plan kind')}
          options={PLAN_MAKES.map((m) => ({
            value: m.value,
            label: m.label,
            hint: m.summary,
            icon: PLAN_ICONS[m.value],
          }))}
          onChange={(value) => (chosen = value)}
        />
        <button
          class="plan-kind-continue"
          type="button"
          onclick={() => {
            choosing = false;
            composing = true;
          }}>{t('Continue')}</button
        >
      </div>
    </OverlaySheet>

    <div id="participation-plans">
      {#if plans.length === 0}
        <div class="phase-empty">
          <strong>{t('No plans yet')}</strong>
          <p>{t('Add a plan when you are ready to propose how this should work.')}</p>
        </div>
      {:else}
        <div class="plan-stack">
          {#each plans as plan (plan.id)}
            <PlanCard
              {plan}
              authorName={detail.names[plan.author] ?? '…'}
              makes={typeless ? planMakesLine(plan.plan.kind) : null}
              canVote={canVoteOn(project)}
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
        <RoundPlusButton
          standout
          active={composing || choosing}
          label={t('Add plan')}
          onclick={add}
        />
      {/if}
    </PhaseWorkToolbar>
  </section>
{/if}

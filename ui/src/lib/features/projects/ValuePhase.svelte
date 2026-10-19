<script lang="ts">
  import PhaseWorkToolbar from '#lib/components/governance/PhaseWorkToolbar.svelte';
  import RoundPlusButton from '#lib/components/RoundPlusButton.svelte';
  import AddValueSheet from '#lib/components/values/AddValueSheet.svelte';
  import ValueCard from '#lib/components/values/ValueCard.svelte';
  import { sortValuesForRating } from '#lib/domain/values.ts';

  import type { DetailState } from './detailState.svelte.ts';

  import '#lib/components/styles/plans.css';
  import '#lib/components/styles/values.css';

  /**
   * web's ProductiveLifecyclePhaseOne (also CollectiveServicePhaseOne): the values the plans
   * will be judged against, rated 1–10 by members, and "Add value" while Proposal is current.
   * Personal services (requests, availability) come with M5.
   */
  let { detail }: { detail: DetailState } = $props();

  let composing = $state(false);

  const project = $derived(detail.project);
  const rankedValues = $derived(sortValuesForRating(detail.values));
  const inValuePhase = $derived(project?.phase === 'phase-1');
  const canAddValue = $derived(inValuePhase && !!project?.amMember);
  const canVoteValues = $derived(inValuePhase && !!project?.amMember);

  async function submit(label: string): Promise<void> {
    if (await detail.addValue(label)) composing = false;
  }
</script>

<div class="value-phase">
  <section class="phase-surface">
    <div id="participation-values" class="surface-stack compact-stack">
      {#if rankedValues.length === 0}
        <div class="phase-empty">
          <strong>No values yet</strong>
          <p>Add the first value this phase should be judged against.</p>
        </div>
      {/if}
      {#each rankedValues as value (value.id)}
        <ValueCard
          {value}
          authorName={detail.names[value.author] ?? '…'}
          canVote={canVoteValues}
          onVote={(importance) => void detail.rateValue(value.id, importance)}
        />
      {/each}
    </div>

    <!-- Always rendered: the toolbar also gives the Phase menu its place in the dock. -->
    <PhaseWorkToolbar>
      {#if canAddValue}
        <RoundPlusButton
          standout
          active={composing}
          label="Add value"
          ariaLabel="Add value proposal"
          onclick={() => (composing = true)}
        />
      {/if}
    </PhaseWorkToolbar>

    {#if canAddValue}
      <AddValueSheet
        open={composing}
        pending={detail.saving}
        message={detail.sheetMessage}
        onSubmit={(label) => void submit(label)}
        onClose={() => (composing = false)}
      />
    {/if}
  </section>
</div>

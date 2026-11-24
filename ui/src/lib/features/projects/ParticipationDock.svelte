<script lang="ts">
  import { canVoteOn } from '#lib/domain/board.ts';
  import DetailActionDock from '#lib/components/DetailActionDock.svelte';
  import VoteDock from '#lib/components/governance/VoteDock.svelte';
  import { isPhaseChange } from '#lib/domain/governance.ts';

  import type { DetailState } from './detailState.svelte.ts';
  import JoinActions from './JoinActions.svelte';

  /**
   * web's Take part dock (ProjectDetailPage's participation DetailActionDock): Join for people
   * who aren't members yet, the phase's work toolbar (`PhaseWorkToolbar` moves in here), and the
   * Vote control for everything but edits and updates (ADR 0099: it floats above the capsule).
   */
  interface Props {
    detail: DetailState;
    active: boolean;
    revealVote?: string | null;
    onRevealDone?: (() => void) | undefined;
  }

  let { detail, active, revealVote = null, onRevealDone }: Props = $props();

  // web's Take part votes: phase moves, a collective service's request settings (ADR 0089),
  // then a software project's (ADR 0086).
  const votes = $derived([
    ...detail.openProposals.filter((p) => isPhaseChange(p.kind)),
    ...detail.openProposals.filter((p) => 'RequestSettings' in p.kind),
    ...detail.software.dockVotes,
  ]);
</script>

{#if detail.project?.charter}
  {@const project = detail.project}
  {@const charter = detail.project.charter}
  <DetailActionDock {active}>
    {#snippet vote()}
      {#if canVoteOn(project)}
        <VoteDock
          proposals={votes}
          names={{ ...detail.names, ...detail.software.names }}
          {charter}
          stage={project.stage}
          phaseLabel={detail.phaseLabel}
          onVote={(p, c) => detail.vote(p, c)}
          newVoter={detail.viewerVotes === false}
          reveal={revealVote}
          {onRevealDone}
        />
      {/if}
    {/snippet}
    {#if !project.amMember}
      <JoinActions {detail} />
    {/if}
    <div id="detail-participation-actions"></div>
  </DetailActionDock>
{/if}

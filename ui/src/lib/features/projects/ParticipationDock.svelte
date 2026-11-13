<script lang="ts">
  import DetailActionDock from '#lib/components/DetailActionDock.svelte';
  import VoteDock from '#lib/components/governance/VoteDock.svelte';
  import MembershipSplitButton from '#lib/components/MembershipSplitButton.svelte';
  import { isPhaseChange } from '#lib/domain/governance.ts';
  import { isEvent } from '#lib/domain/projects.ts';

  import type { DetailState } from './detailState.svelte.ts';

  /**
   * web's Take part dock (ProjectDetailPage's participation DetailActionDock): Join for people
   * who aren't members yet, the phase's work toolbar (`PhaseWorkToolbar` moves in here), and the
   * Vote control for everything but edits and updates (ADR 0099: it floats above the capsule).
   */
  interface Props {
    detail: DetailState;
    active: boolean;
    membersOpen: boolean;
    onOpenMembers: () => void;
    revealVote?: string | null;
    onRevealDone?: (() => void) | undefined;
  }

  let {
    detail,
    active,
    membersOpen,
    onOpenMembers,
    revealVote = null,
    onRevealDone,
  }: Props = $props();

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
      {#if project.amMember}
        <VoteDock
          proposals={votes}
          names={{ ...detail.names, ...detail.software.names }}
          {charter}
          stage={project.stage}
          phaseLabel={detail.phaseLabel}
          onVote={(p, c) => detail.vote(p, c)}
          reveal={revealVote}
          {onRevealDone}
        />
      {/if}
    {/snippet}
    {#if charter.mode !== 'personal-service' && !project.amMember}
      <div class="participation-membership">
        <MembershipSplitButton
          joined={project.amMember}
          count={project.members.length}
          pending={detail.joining}
          {membersOpen}
          noun={isEvent(project.charter?.mode) ? 'event' : 'project'}
          onToggleJoin={() => detail.requestToggleJoin()}
          {onOpenMembers}
        />
      </div>
    {/if}
    <div id="detail-participation-actions"></div>
  </DetailActionDock>
{/if}

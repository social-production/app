<script lang="ts">
  import DetailActionDock from '#lib/components/DetailActionDock.svelte';
  import VoteDock from '#lib/components/governance/VoteDock.svelte';
  import MembershipSplitButton from '#lib/components/MembershipSplitButton.svelte';
  import { kindName } from '#lib/domain/governance.ts';

  import type { DetailState } from './detailState.svelte.ts';

  /**
   * web's Take part dock (ProjectDetailPage's participation DetailActionDock): Join for people
   * who aren't members yet, and the Vote control for everything but edits and updates.
   */
  interface Props {
    detail: DetailState;
    active: boolean;
    membersOpen: boolean;
    onOpenMembers: () => void;
  }

  let { detail, active, membersOpen, onOpenMembers }: Props = $props();

  const votes = $derived(detail.openProposals.filter((p) => kindName(p.kind) === 'phase'));
</script>

{#if detail.project?.charter}
  {@const project = detail.project}
  {@const charter = detail.project.charter}
  <DetailActionDock {active}>
    {#if charter.mode !== 'personal-service' && !project.amMember}
      <div class="participation-membership">
        <MembershipSplitButton
          joined={project.amMember}
          count={project.members.length}
          pending={detail.joining}
          {membersOpen}
          onToggleJoin={() => void detail.toggleJoin()}
          {onOpenMembers}
        />
      </div>
    {/if}
    {#if project.amMember}
      <VoteDock
        proposals={votes}
        names={detail.names}
        {charter}
        stage={project.stage}
        phaseLabel={detail.phaseLabel}
        onVote={(p, c) => detail.vote(p, c)}
      />
    {/if}
  </DetailActionDock>
{/if}

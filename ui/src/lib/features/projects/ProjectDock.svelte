<script lang="ts">
  import SignalStrip from '#lib/components/cards/SignalStrip.svelte';
  import DetailActionDock from '#lib/components/DetailActionDock.svelte';
  import VoteDock from '#lib/components/governance/VoteDock.svelte';
  import MembershipSplitButton from '#lib/components/MembershipSplitButton.svelte';
  import { kindName } from '#lib/domain/governance.ts';
  import { takesSignals } from '#lib/domain/projects.ts';

  import type { DetailState } from './detailState.svelte.ts';

  /** web's Details dock: signals, join/members, Vote on edits and updates, and Edit for members. */
  interface Props {
    detail: DetailState;
    membersOpen: boolean;
    onOpenMembers: () => void;
    onEdit: () => void;
  }

  let { detail, membersOpen, onOpenMembers, onEdit }: Props = $props();
</script>

{#if detail.project?.charter}
  {@const project = detail.project}
  {@const charter = detail.project.charter}
  <DetailActionDock active>
    <div class="project-overview">
      <div class="context-dock">
        {#if takesSignals(charter.mode) && project.phase === 'phase-1'}
          <div id="participation-signals" class="signal-row">
            <SignalStrip
              labeled
              support={project.demand}
              oppose={project.opposition}
              mine={project.mySignal}
              onSignal={(next) => detail.signal(next)}
            />
          </div>
        {/if}
        <div class="control-actions">
          <MembershipSplitButton
            joined={project.amMember}
            count={project.members.length}
            pending={detail.joining}
            {membersOpen}
            onToggleJoin={() => void detail.toggleJoin()}
            {onOpenMembers}
          />
          {#if project.amMember}
            <VoteDock
              proposals={detail.openProposals.filter((p) => kindName(p.kind) !== 'phase')}
              names={detail.names}
              {charter}
              stage={project.stage}
              phaseLabel={detail.phaseLabel}
              onVote={(p, c) => detail.vote(p, c)}
            />
          {/if}
          {#if detail.canPropose}
            <button type="button" class="quiet-control" onclick={onEdit}>Edit</button>
          {/if}
        </div>
      </div>
    </div>
  </DetailActionDock>
{/if}

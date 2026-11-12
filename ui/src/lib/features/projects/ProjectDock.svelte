<script lang="ts">
  import SignalStrip from '#lib/components/cards/SignalStrip.svelte';
  import DetailActionDock from '#lib/components/DetailActionDock.svelte';
  import VoteDock from '#lib/components/governance/VoteDock.svelte';
  import ToolbarIcon from '#lib/components/icons/ToolbarIcon.svelte';
  import MembershipSplitButton from '#lib/components/MembershipSplitButton.svelte';
  import { isPhaseChange } from '#lib/domain/governance.ts';
  import { isEvent, takesSignals } from '#lib/domain/projects.ts';

  import DetailShare from './DetailShare.svelte';
  import type { DetailState } from './detailState.svelte.ts';

  /** web's Details dock: signals, join/members, Vote on edits and updates, Share +, and Edit and
   * Update for members. ADR 0098: Support and Oppose are a capsule above the actions until the
   * person has a stance; then they fold into a chip that opens them again. */
  interface Props {
    detail: DetailState;
    membersOpen: boolean;
    onOpenMembers: () => void;
    onEdit: () => void;
    onUpdate: () => void;
    /** An edit or update a link asks to open (web's `?open=vote&voteKind=edit|update`). */
    revealVote?: string | null;
    onRevealDone?: () => void;
  }

  let {
    detail,
    membersOpen,
    onOpenMembers,
    onEdit,
    onUpdate,
    revealVote = null,
    onRevealDone,
  }: Props = $props();

  let voteOpen = $state(false);
  const project = $derived(detail.project);
  const signals = $derived(
    !!project?.charter && takesSignals(project.charter.mode) && project.phase === 'phase-1',
  );
  const total = $derived((project?.demand ?? 0) + (project?.opposition ?? 0));
  const percent = $derived(
    total > 0 ? `${Math.round(((project?.demand ?? 0) / total) * 100)}%` : '—',
  );
</script>

{#if detail.project?.charter}
  {@const project = detail.project}
  {@const charter = detail.project.charter}
  <DetailActionDock active>
    {#snippet vote()}
      {#if signals && (!project.mySignal || voteOpen)}
        <div id="participation-signals">
          <SignalStrip
            labeled
            bare
            support={project.demand}
            oppose={project.opposition}
            mine={project.mySignal}
            onSignal={async (next) => {
              await detail.signal(next);
              voteOpen = false;
            }}
          />
        </div>
      {/if}
    {/snippet}
    {#if signals && project.mySignal}
      <button
        type="button"
        class="dock-chip"
        class:opposed={project.mySignal === 'Oppose'}
        aria-expanded={voteOpen}
        aria-label={`You ${project.mySignal === 'Oppose' ? 'oppose' : 'support'} this, ${percent} support. Change`}
        title="Change your stance"
        onclick={() => (voteOpen = !voteOpen)}
      >
        <svg aria-hidden="true" viewBox="0 0 24 24" class:flipped={project.mySignal === 'Oppose'}
          ><path
            d="M12 5.5 5 13h4v5.5h6V13h4L12 5.5Z"
            fill="none"
            stroke="currentColor"
            stroke-width="1.8"
            stroke-linejoin="round"
          /></svg
        >
        <span>{percent}</span>
      </button>
    {/if}
    <MembershipSplitButton
      joined={project.amMember}
      count={project.members.length}
      pending={detail.joining}
      {membersOpen}
      noun={isEvent(project.charter?.mode) ? 'event' : 'project'}
      onToggleJoin={() => detail.requestToggleJoin()}
      {onOpenMembers}
    />
    {#if project.amMember}
      <VoteDock
        proposals={detail.openProposals.filter((p) => !isPhaseChange(p.kind))}
        names={detail.names}
        {charter}
        stage={project.stage}
        phaseLabel={detail.phaseLabel}
        onVote={(p, c) => detail.vote(p, c)}
        reveal={revealVote}
        {onRevealDone}
      />
    {/if}
    <DetailShare {detail} />
    {#if detail.canPropose}
      <button type="button" class="dock-icon" aria-label="Edit" title="Edit" onclick={onEdit}>
        <ToolbarIcon name="pencil" />
      </button>
      <button type="button" class="dock-icon" aria-label="Update" title="Update" onclick={onUpdate}>
        <ToolbarIcon name="loudspeaker" />
      </button>
    {/if}
  </DetailActionDock>
{/if}

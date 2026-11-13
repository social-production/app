<script lang="ts">
  import SignalStrip from '#lib/components/cards/SignalStrip.svelte';
  import DetailActionDock from '#lib/components/DetailActionDock.svelte';
  import VoteDock from '#lib/components/governance/VoteDock.svelte';
  import ToolbarIcon from '#lib/components/icons/ToolbarIcon.svelte';
  import MembershipSplitButton from '#lib/components/MembershipSplitButton.svelte';
  import { isPhaseChange } from '#lib/domain/governance.ts';
  import { isEvent, takesSignals, type Stance } from '#lib/domain/projects.ts';

  import DetailShare from './DetailShare.svelte';
  import type { DetailState } from './detailState.svelte.ts';

  /** web's Details dock: signals, join/members, Vote on edits and updates, Share +, and Edit and
   * Update for members. ADR 0099: Support and Oppose are a capsule above the actions until the
   * person has a stance; then they fold into a chip that opens them again. An open vote on
   * an edit or update floats up there too, so the capsule stays one row of actions. */
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
  // The stance just set here, until the project's own read catches up with it (a network
  // read lags a write), so the strip folds into the chip at once.
  let local = $state<{ mine: Stance | null; demand: number; opposition: number } | null>(null);
  const stance = $derived(
    local && local.mine !== (project?.mySignal ?? null)
      ? local
      : {
          mine: project?.mySignal ?? null,
          demand: project?.demand ?? 0,
          opposition: project?.opposition ?? 0,
        },
  );
  const total = $derived(stance.demand + stance.opposition);
  const percent = $derived(total > 0 ? `${Math.round((stance.demand / total) * 100)}%` : '—');

  async function signal(next: Stance | null): Promise<void> {
    const before = stance;
    const count = (s: Stance): number =>
      (s === 'Demand' ? before.demand : before.opposition) -
      Number(before.mine === s) +
      Number(next === s);
    await detail.signal(next);
    local = { mine: next, demand: count('Demand'), opposition: count('Oppose') };
    voteOpen = false;
  }
</script>

{#if detail.project?.charter}
  {@const project = detail.project}
  {@const charter = detail.project.charter}
  <DetailActionDock active>
    {#snippet vote()}
      {#if signals && (!stance.mine || voteOpen)}
        <div id="participation-signals">
          <SignalStrip
            labeled
            bare
            support={stance.demand}
            oppose={stance.opposition}
            mine={stance.mine}
            onSignal={signal}
          />
        </div>
      {/if}
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
    {/snippet}
    {#if signals && stance.mine}
      <button
        type="button"
        class="dock-chip"
        class:opposed={stance.mine === 'Oppose'}
        aria-expanded={voteOpen}
        aria-label={`You ${stance.mine === 'Oppose' ? 'oppose' : 'support'} this, ${percent} support. Change`}
        title="Change your stance"
        onclick={() => (voteOpen = !voteOpen)}
      >
        <svg aria-hidden="true" viewBox="0 0 24 24" class:flipped={stance.mine === 'Oppose'}
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

<script lang="ts">
  import { t } from '#lib/i18n/index.ts';
  import { canVoteOn } from '#lib/domain/board.ts';
  import SignalStrip from '#lib/components/cards/SignalStrip.svelte';
  import DetailActionDock from '#lib/components/DetailActionDock.svelte';
  import VoteDock from '#lib/components/governance/VoteDock.svelte';
  import { isPhaseChange } from '#lib/domain/governance.ts';
  import { isEvent, takesSignals, type Stance } from '#lib/domain/projects.ts';

  import DetailShare from './DetailShare.svelte';
  import type { DetailState } from './detailState.svelte.ts';
  import DockMore from './DockMore.svelte';
  import JoinActions from './JoinActions.svelte';
  import { serviceMenu } from './requests/serviceMenu.svelte.ts';

  /** web's Details dock: signals, Vote on edits and updates, Join, Share, and Update for
   * members. ADR 0099: Support and Oppose are a capsule above the actions until the person
   * has a stance; then they fold into a chip that opens them again. An open vote on an edit or
   * update floats up there too. Detail Pages Cleanup §4: before you join, Join (or "Sign up to
   * join") and Share; once joined, the stance chip, Share and ⋯ (Post an update, Edit, Members,
   * Copy link, Leave; Owner Notes Mockups §10). The member count lives in Details. */
  interface Props {
    detail: DetailState;
    onOpenMembers: () => void;
    onEdit: () => void;
    onUpdate: () => void;
    /** An edit or update a link asks to open (web's `?open=vote&voteKind=edit|update`). */
    revealVote?: string | null;
    onRevealDone?: () => void;
  }

  let {
    detail,
    onOpenMembers,
    onEdit,
    onUpdate,
    revealVote = null,
    onRevealDone,
  }: Props = $props();

  let voteOpen = $state(false);
  const project = $derived(detail.project);
  // Support and Oppose stay on the dock in every phase until it closes, as on web (owner,
  // 2026-11-24); an organizer-run event takes none.
  const signals = $derived(
    !!project?.charter &&
      takesSignals(project.charter.mode) &&
      project.phase !== 'phase-7' &&
      project.governance !== 'organizer_controlled',
  );
  // detail.signal holds the stance you set until the project's read catches up (heldSignal.ts).
  const stance = $derived({
    mine: project?.mySignal ?? null,
    demand: project?.demand ?? 0,
    opposition: project?.opposition ?? 0,
  });
  const total = $derived(stance.demand + stance.opposition);
  const percent = $derived(total > 0 ? `${Math.round((stance.demand / total) * 100)}%` : '—');

  async function signal(next: Stance | null): Promise<void> {
    await detail.signal(next);
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
      {#if canVoteOn(project)}
        <VoteDock
          proposals={detail.openProposals.filter((p) => !isPhaseChange(p.kind))}
          names={detail.names}
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
    {#if signals && stance.mine && project.amMember}
      <button
        type="button"
        class="dock-chip"
        class:opposed={stance.mine === 'Oppose'}
        aria-expanded={voteOpen}
        aria-label={`You ${stance.mine === 'Oppose' ? 'oppose' : 'support'} this, ${percent} support. Change`}
        title={t('Change your stance')}
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
    {#if !project.amMember}
      <JoinActions {detail} />
    {:else}
      <DetailShare {detail} />
      <DockMore
        noun={isEvent(charter.mode) ? 'event' : 'project'}
        canEdit={detail.canPropose}
        {onEdit}
        {onUpdate}
        onSettings={charter.mode === 'collective-service' ? () => (serviceMenu.open = true) : null}
        onMembers={onOpenMembers}
        onLeave={() => detail.requestToggleJoin()}
      />
    {/if}
  </DetailActionDock>
{/if}

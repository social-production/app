<script lang="ts">
  import { t } from '#lib/i18n/index.ts';
  import SkeletonRows from '#lib/components/loading/SkeletonRows.svelte';
  import EmptyState from '#lib/components/empty/EmptyState.svelte';
  import OverlaySheet from '#lib/components/overlay/OverlaySheet.svelte';
  import type { VoteChoice } from '#lib/domain/governance.ts';

  import type { ScopeArt } from './scopeArt.svelte.ts';
  import ScopeDecisionCard from './ScopeDecisionCard.svelte';
  import type { ScopeDecision, ScopeGovernance } from './scopeGovernance.svelte.ts';

  import '#lib/components/styles/scopes.css';

  /**
   * web's ScopeVoteSheet, redrawn (Channel Pictures mockups §3): the open decisions you
   * haven't voted on, one card at a time with "1 of N"; voting moves to the next. Votes are
   * final here (ADR 0033), so there is no changing one afterwards.
   */
  interface Props {
    open: boolean;
    gov: ScopeGovernance;
    memberName: (agent: string) => string;
    art?: ScopeArt | undefined;
    onClose: () => void;
  }

  let { open, gov, memberName, art, onClose }: Props = $props();

  let index = $state(0);
  let voted = $state<VoteChoice | null>(null);
  let notice = $state('');

  $effect(() => {
    if (!open) return;
    index = 0;
    voted = null;
    notice = '';
  });

  // While the "You voted…" line shows, keep the decision just voted on in view.
  let shown = $state<ScopeDecision | null>(null);
  const votable = $derived(gov.votable);
  const current = $derived(shown ?? votable[index] ?? null);

  async function cast(item: ScopeDecision, choice: VoteChoice): Promise<void> {
    shown = item;
    if (!(await gov.vote(item.id, choice))) {
      shown = null;
      return;
    }
    voted = choice;
    const stillOpen = gov.open.some((p) => p.id === item.id);
    notice = stillOpen ? t('Vote recorded') : t('Vote recorded. That decision is now settled.');
    // Leave "You voted…" on screen for a moment before moving on.
    await new Promise((resolve) => setTimeout(resolve, 1200));
    shown = null;
    voted = null;
    index = 0;
    if (gov.votable.length === 0) onClose();
  }
</script>

<OverlaySheet
  {open}
  title={t('Vote')}
  labelledById="scope-vote-title"
  rootClass="scope-vote"
  {onClose}
>
  <div class="vote-body">
    {#if gov.message}<p class="meta">{gov.message}</p>{/if}
    {#if current}
      <p class="vote-count">{index + 1} of {votable.length || 1}</p>
      <ScopeDecisionCard
        item={current}
        {memberName}
        {art}
        busy={gov.busy}
        chosen={voted}
        onVote={(choice) => current && void cast(current, choice)}
      />
      {#if voted}<p class="voted">{t('You voted {choice}.', { choice: t(voted) })} {notice}</p>{/if}
      {#if votable.length > 1 && !shown}
        <button class="next" type="button" onclick={() => (index = (index + 1) % votable.length)}
          >{t('Next')}</button
        >
      {/if}
    {:else if !gov.loaded}
      <SkeletonRows count={2} kind="row" />
    {:else}
      <EmptyState
        icon="check"
        title={t('No open votes')}
        text={t('New proposals for this space show up here.')}
      />
    {/if}
  </div>
</OverlaySheet>

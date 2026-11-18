<script lang="ts">
  import { t } from '#lib/i18n/index.ts';
  import { APPROVAL_PERCENT, type VoteChoice } from '#lib/domain/governance.ts';

  import { imageUrl, type ScopeArt } from './scopeArt.svelte.ts';
  import { describeDecision, type ScopeDecision } from './scopeGovernance.svelte.ts';

  /**
   * One scope decision as a card (Channel Pictures mockups §3): what it changes with its tag,
   * the new text, a bar that fills towards the votes needed, one line with the count and who
   * proposed it, and No / Yes while it is open and yours to vote on.
   */
  interface Props {
    item: ScopeDecision;
    memberName: (agent: string) => string;
    /** Shows No / Yes; votes are final (ADR 0033). */
    onVote?: ((choice: VoteChoice) => void) | undefined;
    busy?: boolean;
    /** The choice just made, kept marked while the sheet moves on. */
    chosen?: VoteChoice | null;
    /** The scope's current picture and cover, shown as "Now" beside a proposed one. */
    art?: ScopeArt | undefined;
  }

  let { item, memberName, onVote, busy = false, chosen = null, art }: Props = $props();

  // A picture or cover change shows Now and Proposed side by side (Channel Pictures §3).
  const change = $derived.by(() => {
    if (!('Edit' in item.kind)) return null;
    const { picture, cover } = item.kind.Edit;
    const pick = cover ?? picture;
    if (!pick) return null;
    return {
      slot: cover ? ('cover' as const) : ('picture' as const),
      hash: pick === 'Remove' ? null : pick.Set,
    };
  });
  let proposedUrl = $state<string | null>(null);
  $effect(() => {
    const hash = change?.hash ?? null;
    void imageUrl(hash).then((u) => (proposedUrl = u));
  });
  const nowUrl = $derived(
    change ? (change.slot === 'cover' ? art?.coverUrl : art?.pictureUrl) : null,
  );

  const open = $derived(item.status === 'Open');
  const share = $derived(
    item.yes + item.no > 0 ? Math.round((item.yes / (item.yes + item.no)) * 100) : 0,
  );
  const fill = $derived(
    open ? Math.min(100, Math.round((item.yes / Math.max(item.required, 1)) * 100)) : share,
  );
  const tag = $derived(
    open ? 'Open' : `${item.status === 'Passed' ? 'Passed' : 'Rejected'} · ${share}% yes`,
  );
  const text = $derived('Edit' in item.kind ? item.kind.Edit.description : null);
</script>

<article
  class="decision-card"
  class:open
  class:approved={item.status === 'Passed'}
  class:rejected={!open && item.status !== 'Passed'}
>
  <div class="decision-top">
    <strong>{describeDecision(item, memberName)}</strong>
    <span class="decision-tag">{tag}</span>
  </div>
  {#if text}<p class="decision-text">"{text}"</p>{/if}
  {#if change}
    <div class="decision-art" class:square={change.slot === 'picture'}>
      {#if open}
        <figure>
          <span style={nowUrl ? `background-image: url(${nowUrl})` : undefined}></span>
          <figcaption>{t('Now')}</figcaption>
        </figure>
        <span class="decision-art-arrow" aria-hidden="true">›</span>
      {/if}
      <figure>
        <span style={proposedUrl ? `background-image: url(${proposedUrl})` : undefined}></span>
        <figcaption>{change.hash ? t('Proposed') : t('Removed')}</figcaption>
      </figure>
    </div>
  {/if}
  <div class="decision-bar" aria-hidden="true"><i style={`width: ${Math.max(fill, 3)}%`}></i></div>
  <p class="decision-line">
    {item.yes} yes · {item.no} no · {item.required} votes needed, {APPROVAL_PERCENT}% yes · by {item.proposer}
  </p>
  {#if onVote}
    <div class="decision-choices">
      <button
        class="no"
        type="button"
        class:chosen={chosen === 'No'}
        aria-pressed={chosen === 'No'}
        disabled={busy || chosen !== null}
        onclick={() => onVote('No')}>{t('No')}</button
      >
      <button
        class="yes"
        type="button"
        class:chosen={chosen === 'Yes'}
        aria-pressed={chosen === 'Yes'}
        disabled={busy || chosen !== null}
        onclick={() => onVote('Yes')}>{t('Yes')}</button
      >
    </div>
  {/if}
</article>

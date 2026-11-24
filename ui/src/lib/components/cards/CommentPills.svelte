<script lang="ts">
  import { t, plural } from '#lib/i18n/index.ts';
  import type { Snippet } from 'svelte';

  import { shareLink } from '#lib/components/overlay/shareLink.ts';
  import { formatCompactCount } from '#lib/domain/count.ts';
  import { directionOf, stanceOf, type VoteDirection } from '#lib/domain/discussions.ts';

  import VoteStrip from './VoteStrip.svelte';

  import '#lib/components/styles/ux/item.css';

  /**
   * A comment's actions, the same soft pills as its thread's or post's own row (owner,
   * 2026-11-24): ▲ score ▼, the replies, Share; then whatever the caller adds (⋮ report).
   * In the thread the replies pill answers the comment; elsewhere it opens it.
   */
  interface Props {
    score: number;
    mine: VoteDirection;
    onVote: (direction: VoteDirection) => Promise<void>;
    replies: number;
    onReply?: () => void;
    replyHref?: string;
    sharePath: string;
    shareTitle: string;
    children?: Snippet;
  }

  let { score, mine, onVote, replies, onReply, replyHref, sharePath, shareTitle, children }: Props =
    $props();

  const sheet = shareLink();
  const label = $derived(plural(replies, '{n} reply', '{n} replies'));

  function share(event: MouseEvent): void {
    event.preventDefault();
    event.stopPropagation();
    sheet?.open({ kind: 'comment', title: shareTitle, path: sharePath });
  }
</script>

{#snippet bubble()}
  <svg aria-hidden="true" viewBox="0 0 24 24" class="feed-action-icon"
    ><path
      d="M6 5.5h12a1.5 1.5 0 0 1 1.5 1.5v8a1.5 1.5 0 0 1-1.5 1.5h-8l-4 3v-3A1.5 1.5 0 0 1 4.5 15V7A1.5 1.5 0 0 1 6 5.5Z"
      fill="none"
      stroke="currentColor"
      stroke-width="1.8"
      stroke-linejoin="round"
    /></svg
  >
  <span aria-hidden="true">{formatCompactCount(replies)}</span>
{/snippet}

<div class="item-actions comment-pills">
  <VoteStrip
    bare
    support={score}
    oppose={0}
    mine={stanceOf(mine)}
    onSignal={(next) => onVote(directionOf(next))}
  />
  {#if onReply}
    <button
      class="item-pill"
      type="button"
      aria-label={`${t('Reply')} · ${label}`}
      onclick={(e) => (e.stopPropagation(), onReply())}>{@render bubble()}</button
    >
  {:else}
    <!-- eslint-disable-next-line svelte/no-navigation-without-resolve -- the caller resolves it -->
    <a class="item-pill" href={replyHref} aria-label={label}>{@render bubble()}</a>
  {/if}
  {#if sheet}
    <button class="share-button" type="button" onclick={share}>
      <svg aria-hidden="true" viewBox="0 0 24 24"
        ><path
          d="M12 15V4.5M8 8l4-3.5L16 8M6 12.5V18a1 1 0 0 0 1 1h10a1 1 0 0 0 1-1v-5.5"
          fill="none"
          stroke="currentColor"
          stroke-width="1.8"
          stroke-linecap="round"
          stroke-linejoin="round"
        /></svg
      >
      <span>{t('Share')}</span>
    </button>
  {/if}
  {@render children?.()}
</div>

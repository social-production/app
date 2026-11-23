<script lang="ts">
  import { t } from '#lib/i18n/index.ts';
  import { shareLink } from '#lib/components/overlay/shareLink.ts';
  import AvatarBadge from '#lib/components/icons/AvatarBadge.svelte';
  import PausedBadge from '#lib/components/people/PausedBadge.svelte';
  import { bodyParts } from '#lib/components/chat/chatFormat.ts';
  import ReportControl from '#lib/components/moderation/ReportControl.svelte';
  import type { DiscussionComment as Comment, VoteDirection } from '#lib/domain/discussions.ts';
  import { VISIBLE, removedLabel, type Moderation } from '#lib/domain/moderation.ts';
  import { formatCompactCount } from '#lib/domain/count.ts';
  import { relativeCompact } from '#lib/domain/time.ts';

  import { resolve } from '$app/paths';

  import type { CommentMentions } from './commentMentions.ts';
  import Self from './DiscussionComment.svelte';

  /**
   * web's DiscussionComment: author and time (the row folds the comment and its replies), the
   * body (or web's "Removed for …" and serious-harm reveal, ADR 0071 part b), Like, Reply and
   * Share with their counts (owner, 2026-11-24) and the ⋮ report menu, then the replies. Reply hands the page's one composer to this
   * comment, which is outlined while it is answered (Profiles and Comments mockups §4).
   */
  interface Props {
    comment: Comment;
    /** The thread's or post's listing id (its reports' space). */
    space: string;
    names: Record<string, string>;
    highlighted: string | null;
    embedded?: boolean;
    /** Posts (ADR 0100): an avatar column, replies at the same level under their parent. */
    flat?: boolean;
    /** Flat replies name whose comment they answer. */
    replyingTo?: string | null;
    onVote: (id: string, direction: VoteDirection) => Promise<void>;
    /** The comment the composer answers now (outlined). */
    replyTarget?: string | null;
    /** Reply: the composer answers this comment (by its author). */
    onReply: (parent: string, author: string) => void;
    /** A report or vote changed this comment's state. */
    onModeration: (id: string, next: Moderation) => void;
    mentions?: CommentMentions | null;
  }

  let {
    comment,
    space,
    names,
    highlighted,
    embedded = false,
    flat = false,
    replyingTo = null,
    replyTarget = null,
    onVote,
    onReply,
    onModeration,
    mentions = null,
  }: Props = $props();

  let collapsed = $state(false);
  let revealed = $state(false);
  const moderation = $derived(comment.moderation ?? VISIBLE);
  // web's shouldHideModeratedBody and supportsHiddenToggle.
  const removed = $derived(moderation.state === 'removed');
  const canReveal = $derived(
    !removed && (moderation.report?.reason === 'serious-harm' || moderation.state === 'hidden'),
  );
  let self = $state<HTMLElement>();
  const isHighlighted = $derived(highlighted === comment.id);
  const name = $derived(names[comment.author] ?? '…');
  // Like, replies and share, as a tweet's reply row (owner, 2026-11-24).
  const liked = $derived(comment.activeVote === 1);
  const likes = $derived(Math.max(0, comment.voteCount));
  const replyCount = $derived(countReplies(comment));
  const sheet = shareLink();

  function countReplies(c: Comment): number {
    return c.replies.reduce((n, r) => n + 1 + countReplies(r), 0);
  }

  function share(): void {
    const path = `${location.pathname}?comment=${encodeURIComponent(comment.id)}`;
    const title = comment.body.length > 80 ? `${comment.body.slice(0, 79)}…` : comment.body;
    sheet?.open({ kind: 'comment', title: `${name}: ${title}`, path });
  }

  $effect(() => {
    if (isHighlighted)
      setTimeout(() => self?.scrollIntoView({ block: 'center', behavior: 'smooth' }), 150);
  });
</script>

<article
  id={`comment-${comment.id}`}
  class:embedded
  class:flat
  class:has-replies={comment.replies.length > 0}
  class:answering={replyTarget === comment.id}
  class:sending={comment.sending}
  class="comment-card"
>
  <div
    bind:this={self}
    class:highlighted={isHighlighted}
    class:reply-target={replyTarget === comment.id}
    class="comment-self"
  >
    <span class="comment-avatar"><AvatarBadge size="sm" {name} agent={comment.author} /></span>
    <div class="topline">
      <a class="author-link" href={resolve('/profile/[agent]', { agent: comment.author })}>
        {name}
      </a>
      <PausedBadge agent={comment.author} />
      <span class="comment-time">{relativeCompact(comment.createdAt)}</span>
      {#if comment.sending}<span class="comment-sending">{t('Sending…')}</span>{/if}
      <button
        aria-expanded={!collapsed}
        aria-label={collapsed
          ? t('Show this comment and its replies')
          : t('Hide this comment and its replies')}
        class="collapse-hit"
        type="button"
        onclick={() => (collapsed = !collapsed)}
      >
        {#if collapsed}<span class="collapsed-mark">+</span>{/if}
      </button>
    </div>

    <div class="comment-body-block" hidden={collapsed}>
      {#if canReveal}
        <button
          aria-expanded={revealed}
          class="hidden-toggle"
          type="button"
          onclick={() => (revealed = !revealed)}
        >
          <span class="hidden-plus">{revealed ? '−' : '+'}</span>
          <span
            >{revealed
              ? t('Hide again')
              : t('Hidden for serious harm report — reveal to read')}</span
          >
        </button>
      {/if}
      {#if flat && replyingTo}
        <p class="replying-to">{t('Replying to')} <em>{replyingTo}</em></p>
      {/if}
      {#if removed}
        <p class="body moderated">{t(removedLabel(moderation.report?.reason))}</p>
      {:else if !canReveal || revealed}
        <p class="body">
          {#each bodyParts(comment.body) as part, i (i)}
            {#if part.mention}
              {@const agent = mentions?.agentOf(part.mention)}
              {#if agent}
                <a class="mention" href={resolve('/profile/[agent]', { agent })}>{part.text}</a>
              {:else}<span class="mention">{part.text}</span>{/if}
            {:else if part.internal}
              <!-- eslint-disable-next-line svelte/no-navigation-without-resolve -- a path written in the comment -->
              <a href={part.href}>{part.text}</a>
            {:else if part.href}
              <!-- eslint-disable-next-line svelte/no-navigation-without-resolve -- external link -->
              <a href={part.href} target="_blank" rel="noopener noreferrer">{part.text}</a>
            {:else}<span class="body-text">{part.text}</span>{/if}
          {/each}
        </p>
      {/if}

      {#if !comment.sending}
        <div class="actions-row">
          <button
            class="comment-act like"
            class:on={liked}
            type="button"
            aria-pressed={liked}
            aria-label={liked ? t('Unlike') : t('Like')}
            onclick={() => onVote(comment.id, liked ? 0 : 1)}
          >
            <svg aria-hidden="true" viewBox="0 0 24 24">
              <path
                d="M12 20s-7-4.4-7-10a4 4 0 0 1 7-2.6A4 4 0 0 1 19 10c0 5.6-7 10-7 10z"
                fill={liked ? 'currentColor' : 'none'}
                stroke="currentColor"
                stroke-width="1.8"
                stroke-linejoin="round"
              />
            </svg>
            {#if likes > 0}<span>{formatCompactCount(likes)}</span>{/if}
          </button>
          <button
            class="comment-act reply-button"
            type="button"
            aria-label={t('Reply')}
            onclick={() => onReply(comment.id, comment.author)}
          >
            <svg aria-hidden="true" viewBox="0 0 24 24">
              <path
                d="M6 5.5h12a1.5 1.5 0 0 1 1.5 1.5v8a1.5 1.5 0 0 1-1.5 1.5h-8l-4 3v-3A1.5 1.5 0 0 1 4.5 15V7A1.5 1.5 0 0 1 6 5.5Z"
                fill="none"
                stroke="currentColor"
                stroke-width="1.8"
                stroke-linejoin="round"
              />
            </svg>
            {#if replyCount > 0}<span>{formatCompactCount(replyCount)}</span>{/if}
          </button>
          {#if sheet}
            <button class="comment-act" type="button" aria-label={t('Share')} onclick={share}>
              <svg aria-hidden="true" viewBox="0 0 24 24">
                <path
                  d="M12 15V4.5M8 8l4-3.5L16 8M6 12.5V18a1 1 0 0 0 1 1h10a1 1 0 0 0 1-1v-5.5"
                  fill="none"
                  stroke="currentColor"
                  stroke-width="1.8"
                  stroke-linecap="round"
                  stroke-linejoin="round"
                />
              </svg>
            </button>
          {/if}
          <ReportControl
            id={space}
            message={comment.id}
            itemLabel="comment"
            {moderation}
            owner={comment.author}
            onChange={(next) => onModeration(comment.id, next)}
          />
        </div>
      {/if}
    </div>
  </div>

  {#if comment.replies.length > 0}
    <div class="reply-stack" hidden={collapsed}>
      {#each comment.replies as child (child.id)}
        <Self
          comment={child}
          {space}
          {names}
          {highlighted}
          {embedded}
          {flat}
          replyingTo={flat ? name : null}
          {replyTarget}
          {onVote}
          {onReply}
          {onModeration}
          {mentions}
        />
      {/each}
    </div>
  {/if}
</article>

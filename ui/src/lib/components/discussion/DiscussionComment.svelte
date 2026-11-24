<script lang="ts">
  import { t } from '#lib/i18n/index.ts';
  import CommentPills from '#lib/components/cards/CommentPills.svelte';
  import AvatarBadge from '#lib/components/icons/AvatarBadge.svelte';
  import PausedBadge from '#lib/components/people/PausedBadge.svelte';
  import { bodyParts } from '#lib/components/chat/chatFormat.ts';
  import ReportControl from '#lib/components/moderation/ReportControl.svelte';
  import type { DiscussionComment as Comment, VoteDirection } from '#lib/domain/discussions.ts';
  import { VISIBLE, removedLabel, type Moderation } from '#lib/domain/moderation.ts';
  import { relativeCompact } from '#lib/domain/time.ts';

  import { resolve } from '$app/paths';

  import type { CommentMentions } from './commentMentions.ts';
  import Self from './DiscussionComment.svelte';

  /**
   * web's DiscussionComment: author and time (the row folds the comment and its replies), the
   * body (or web's "Removed for …" and serious-harm reveal, ADR 0071 part b), the feed rows'
   * replies · votes · share row (owner, 2026-11-24) and the ⋮ report menu, then the replies. Reply hands the page's one composer to this
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
    /** Posts (owner, 2026-11-24): replies live in the comment's own view, which tapping the
     * comment opens; its replies are not listed under it here. */
    onOpen?: ((id: string) => void) | undefined;
    /** List no replies under it (the opened comment, or one of its parents). */
    hideReplies?: boolean;
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
    onOpen,
    hideReplies = false,
  }: Props = $props();

  // Tapping the comment's words opens it; links and buttons inside keep their own taps.
  function open(event: Event): void {
    if (!onOpen || (event.target as HTMLElement).closest('a, button')) return;
    onOpen(comment.id);
  }

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
  // The feed rows' action row: replies, votes, share (owner, 2026-11-24).
  const replyCount = $derived(countReplies(comment));
  const sharePath = $derived(`${location.pathname}?comment=${encodeURIComponent(comment.id)}`);
  const shareTitle = $derived(
    `${name}: ${comment.body.length > 80 ? `${comment.body.slice(0, 79)}…` : comment.body}`,
  );

  function countReplies(c: Comment): number {
    return c.replies.reduce((n, r) => n + 1 + countReplies(r), 0);
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
        <!-- svelte-ignore a11y_no_noninteractive_tabindex -->
        <p
          class="body"
          class:opens={!!onOpen}
          role={onOpen ? 'link' : undefined}
          tabindex={onOpen ? 0 : undefined}
          onclick={open}
          onkeydown={(e) => e.key === 'Enter' && open(e)}
        >
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
        <CommentPills
          score={comment.voteCount}
          mine={comment.activeVote}
          onVote={(direction) => onVote(comment.id, direction)}
          replies={replyCount}
          onReply={() => onReply(comment.id, comment.author)}
          {sharePath}
          {shareTitle}
        >
          <ReportControl
            id={space}
            message={comment.id}
            itemLabel="comment"
            {moderation}
            owner={comment.author}
            onChange={(next) => onModeration(comment.id, next)}
          />
        </CommentPills>
      {/if}
    </div>
  </div>

  {#if comment.replies.length > 0 && !onOpen && !hideReplies}
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

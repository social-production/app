<script lang="ts">
  import VoteStrip from '#lib/components/cards/VoteStrip.svelte';
  import PausedBadge from '#lib/components/people/PausedBadge.svelte';
  import { bodyParts } from '#lib/components/chat/chatFormat.ts';
  import ReportControl from '#lib/components/moderation/ReportControl.svelte';
  import type { DiscussionComment as Comment, VoteDirection } from '#lib/domain/discussions.ts';
  import { VISIBLE, removedLabel, type Moderation } from '#lib/domain/moderation.ts';
  import type { Stance } from '#lib/domain/projects.ts';
  import { relativeCompact } from '#lib/domain/time.ts';

  import { resolve } from '$app/paths';

  import type { CommentMentions } from './commentMentions.ts';
  import CommentComposer from './CommentComposer.svelte';
  import Self from './DiscussionComment.svelte';

  /**
   * web's DiscussionComment: author and time (the row folds the comment and its replies), the
   * body (or web's "Removed for …" and serious-harm reveal, ADR 0071 part b), a vote strip,
   * Reply and the ⋮ report menu, a reply composer, then the replies.
   */
  interface Props {
    comment: Comment;
    /** The thread's or post's listing id (its reports' space). */
    space: string;
    names: Record<string, string>;
    highlighted: string | null;
    embedded?: boolean;
    signedIn: boolean;
    onSignIn: (message: string) => void;
    onVote: (id: string, direction: VoteDirection) => Promise<void>;
    /** Resolves true once the reply is written (the draft then clears). */
    onReply: (parent: string, body: string) => Promise<boolean>;
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
    signedIn,
    onSignIn,
    onVote,
    onReply,
    onModeration,
    mentions = null,
  }: Props = $props();

  let draft = $state('');
  let replyOpen = $state(false);
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
  const stance = $derived<Stance | null>(
    comment.activeVote === 1 ? 'Demand' : comment.activeVote === -1 ? 'Oppose' : null,
  );

  $effect(() => {
    if (isHighlighted) self?.scrollIntoView({ block: 'center' });
  });

  async function reply(): Promise<void> {
    if (await onReply(comment.id, draft)) {
      draft = '';
      replyOpen = false;
    }
  }
</script>

<article
  id={`comment-${comment.id}`}
  class:embedded
  class:reply-open={replyOpen}
  class="comment-card"
>
  <div bind:this={self} class:highlighted={isHighlighted} class="comment-self">
    <div class="topline">
      <a class="author-link" href={resolve('/profile/[agent]', { agent: comment.author })}>
        {name}
      </a>
      <PausedBadge agent={comment.author} />
      <span class="comment-time">{relativeCompact(comment.createdAt)}</span>
      <button
        aria-expanded={!collapsed}
        aria-label={collapsed
          ? 'Show this comment and its replies'
          : 'Hide this comment and its replies'}
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
          <span>{revealed ? 'Hide again' : 'Hidden for serious harm report — reveal to read'}</span>
        </button>
      {/if}
      {#if removed}
        <p class="body moderated">{removedLabel(moderation.report?.reason)}</p>
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
            {:else}{part.text}{/if}
          {/each}
        </p>
      {/if}

      <div class="actions-row">
        <VoteStrip
          support={comment.voteCount}
          oppose={0}
          mine={stance}
          onSignal={(next) =>
            onVote(comment.id, next === 'Demand' ? 1 : next === 'Oppose' ? -1 : 0)}
        />
        <button
          class="reply-button"
          type="button"
          aria-label={replyOpen ? 'Cancel reply' : 'Reply'}
          onclick={() => (replyOpen = !replyOpen)}
        >
          {#if replyOpen}
            <span aria-hidden="true">×</span>
          {:else}
            <svg aria-hidden="true" viewBox="0 0 24 24">
              <path
                d="M9 10 4 15l5 5"
                fill="none"
                stroke="currentColor"
                stroke-width="2"
                stroke-linecap="round"
                stroke-linejoin="round"
              />
              <path
                d="M20 4v7a4 4 0 0 1-4 4H4"
                fill="none"
                stroke="currentColor"
                stroke-width="2"
                stroke-linecap="round"
              />
            </svg>
          {/if}
        </button>
        <ReportControl
          id={space}
          message={comment.id}
          itemLabel="comment"
          {moderation}
          owner={comment.author}
          onChange={(next) => onModeration(comment.id, next)}
        />
      </div>

      {#if replyOpen}
        <div class="composer-card">
          <CommentComposer
            bind:value={draft}
            placeholder="Write a reply..."
            submitLabel="Post reply"
            {signedIn}
            {onSignIn}
            mentionSearch={mentions?.search}
            onSubmit={() => void reply()}
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
          {signedIn}
          {onSignIn}
          {onVote}
          {onReply}
          {onModeration}
          {mentions}
        />
      {/each}
    </div>
  {/if}
</article>

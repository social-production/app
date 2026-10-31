<script lang="ts">
  import VoteStrip from '#lib/components/cards/VoteStrip.svelte';
  import { bodyParts } from '#lib/components/chat/chatFormat.ts';
  import type { DiscussionComment as Comment, VoteDirection } from '#lib/domain/discussions.ts';
  import type { Stance } from '#lib/domain/projects.ts';
  import { relativeCompact } from '#lib/domain/time.ts';

  import { resolve } from '$app/paths';

  import CommentComposer from './CommentComposer.svelte';
  import Self from './DiscussionComment.svelte';

  /**
   * web's DiscussionComment: author and time (the row folds the comment and its replies), the
   * body, a vote strip and Reply, a reply composer, then the replies. web's ⋮ Report menu and
   * moderated bodies come with moderation.
   */
  interface Props {
    comment: Comment;
    names: Record<string, string>;
    highlighted: string | null;
    embedded?: boolean;
    signedIn: boolean;
    onSignIn: (message: string) => void;
    onVote: (id: string, direction: VoteDirection) => Promise<void>;
    /** Resolves true once the reply is written (the draft then clears). */
    onReply: (parent: string, body: string) => Promise<boolean>;
  }

  let {
    comment,
    names,
    highlighted,
    embedded = false,
    signedIn,
    onSignIn,
    onVote,
    onReply,
  }: Props = $props();

  let draft = $state('');
  let replyOpen = $state(false);
  let collapsed = $state(false);
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
      <p class="body">
        {#each bodyParts(comment.body) as part, i (i)}
          {#if part.mention}<span class="mention">{part.text}</span>
          {:else if part.internal}
            <!-- eslint-disable-next-line svelte/no-navigation-without-resolve -- a path written in the comment -->
            <a href={part.href}>{part.text}</a>
          {:else if part.href}
            <!-- eslint-disable-next-line svelte/no-navigation-without-resolve -- external link -->
            <a href={part.href} target="_blank" rel="noopener noreferrer">{part.text}</a>
          {:else}{part.text}{/if}
        {/each}
      </p>

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
      </div>

      {#if replyOpen}
        <div class="composer-card">
          <CommentComposer
            bind:value={draft}
            placeholder="Write a reply..."
            submitLabel="Post reply"
            {signedIn}
            {onSignIn}
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
          {names}
          {highlighted}
          {embedded}
          {signedIn}
          {onSignIn}
          {onVote}
          {onReply}
        />
      {/each}
    </div>
  {/if}
</article>

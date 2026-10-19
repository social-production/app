<script lang="ts">
  import VoteStrip from '#lib/components/cards/VoteStrip.svelte';
  import { bodyParts } from '#lib/components/chat/chatFormat.ts';
  import { relativeCompact } from '#lib/domain/time.ts';
  import { COMMENT_MAX, type CommentNode, type VoteDirection } from '#lib/domain/threads.ts';

  import { resolve } from '$app/paths';

  import CommentComposer from './CommentComposer.svelte';
  import DiscussionComment from './DiscussionComment.svelte';

  import '#lib/components/styles/discussion.css';

  /**
   * web's DiscussionComment: author, time, body (http(s) links only), votes, reply, and the
   * replies beneath; tapping the top line folds the chain. Reports arrive with moderation (M5).
   */
  interface Props {
    comment: CommentNode;
    embedded?: boolean;
    paused?: boolean;
    onVote: (id: string, next: VoteDirection) => Promise<void>;
    /** Resolves true once the reply is written, so the draft can clear. */
    onReply: (parent: string, body: string) => Promise<boolean>;
  }

  let { comment, embedded = false, paused = false, onVote, onReply }: Props = $props();

  let draft = $state('');
  let replyOpen = $state(false);
  let collapsed = $state(false);
  let composer = $state<CommentComposer>();

  async function submitReply(): Promise<void> {
    if (!draft.trim()) return;
    if (!(await onReply(comment.id, draft))) return;
    draft = '';
    replyOpen = false;
    await composer?.resetHeight();
  }
</script>

<article
  id={`comment-${comment.id}`}
  class="comment-card"
  class:embedded
  class:reply-open={replyOpen}
>
  <div class="comment-self">
    <div class="topline">
      <a class="author-link" href={resolve('/profile/[agent]', { agent: comment.author })}
        >{comment.authorName}</a
      >
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
          {#if part.href}
            <!-- eslint-disable-next-line svelte/no-navigation-without-resolve -- external link -->
            <a href={part.href} target="_blank" rel="noopener noreferrer">{part.text}</a>
          {:else}{part.text}{/if}
        {/each}
      </p>

      <div class="actions-row">
        <VoteStrip
          count={comment.voteCount}
          active={comment.myVote}
          onVote={(next) => onVote(comment.id, next)}
        />
        {#if !paused}
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
        {/if}
      </div>

      {#if replyOpen}
        <div class="composer-card">
          <CommentComposer
            bind:this={composer}
            bind:value={draft}
            placeholder="Write a reply..."
            submitLabel="Post reply"
            maxLength={COMMENT_MAX}
            onSubmit={() => void submitReply()}
          />
        </div>
      {/if}
    </div>
  </div>

  {#if comment.replies.length > 0}
    <div class="reply-stack" hidden={collapsed}>
      {#each comment.replies as reply (reply.id)}
        <DiscussionComment comment={reply} {embedded} {paused} {onVote} {onReply} />
      {/each}
    </div>
  {/if}
</article>

<script lang="ts">
  import { COMMENT_MAX, type CommentNode, type VoteDirection } from '#lib/domain/threads.ts';

  import CommentComposer from './CommentComposer.svelte';
  import DiscussionComment from './DiscussionComment.svelte';

  import '#lib/components/styles/discussion.css';

  /**
   * web's DiscussionPanel: the comment composer (pinned above the bottom nav on phones), then
   * the comments, already nested and sorted. Paused accounts get the read-only notice.
   */
  interface Props {
    comments: CommentNode[];
    paused: boolean;
    error?: string;
    embedded?: boolean;
    /** Resolves true once the comment is written, so the draft can clear. */
    onComment: (body: string) => Promise<boolean>;
    onReply: (parent: string, body: string) => Promise<boolean>;
    onVote: (id: string, next: VoteDirection) => Promise<void>;
  }

  let {
    comments,
    paused,
    error = '',
    embedded = false,
    onComment,
    onReply,
    onVote,
  }: Props = $props();

  let draft = $state('');
  let composer = $state<CommentComposer>();

  async function submit(): Promise<void> {
    if (!draft.trim()) return;
    if (!(await onComment(draft))) return;
    draft = '';
    await composer?.resetHeight();
  }
</script>

<section class="discussion-shell" class:embedded id="comments">
  {#if paused}
    <p class="composer-paused" role="status">
      This account is read-only: a community review found it's a bot.
    </p>
  {:else}
    <div class="composer-card">
      <CommentComposer
        bind:this={composer}
        bind:value={draft}
        placeholder="Write a comment..."
        submitLabel="Post comment"
        maxLength={COMMENT_MAX}
        onSubmit={() => void submit()}
      />
    </div>
  {/if}
  {#if error}<p class="discussion-error" role="alert">{error}</p>{/if}

  <div class="stack">
    {#each comments as comment (comment.id)}
      <DiscussionComment {comment} {embedded} {paused} {onVote} {onReply} />
    {/each}
  </div>
</section>

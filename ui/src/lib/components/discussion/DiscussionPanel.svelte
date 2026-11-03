<script lang="ts">
  import {
    sortComments,
    type CommentSort,
    type DiscussionComment as Comment,
    type VoteDirection,
  } from '#lib/domain/discussions.ts';
  import type { Moderation } from '#lib/domain/moderation.ts';

  import CommentComposer from './CommentComposer.svelte';
  import DiscussionComment from './DiscussionComment.svelte';

  import '#lib/components/styles/discussion.css';

  /**
   * web's DiscussionPanel: the comment composer (docked at the bottom on phones) and the
   * comments, top-level ones in the chosen order (ADR 0070).
   */
  interface Props {
    comments: Comment[];
    /** The thread's or post's listing id. */
    space: string;
    names: Record<string, string>;
    sortMode: CommentSort;
    highlighted: string | null;
    embedded?: boolean;
    signedIn: boolean;
    onSignIn: (message: string) => void;
    onVote: (id: string, direction: VoteDirection) => Promise<void>;
    /** Writes a comment (`parent` null) or a reply; resolves true once written. */
    onComment: (parent: string | null, body: string) => Promise<boolean>;
    onModeration: (id: string, next: Moderation) => void;
  }

  let {
    comments,
    space,
    names,
    sortMode,
    highlighted,
    embedded = false,
    signedIn,
    onSignIn,
    onVote,
    onComment,
    onModeration,
  }: Props = $props();

  let draft = $state('');
  const sorted = $derived(sortComments(comments, sortMode));

  async function submit(): Promise<void> {
    if (await onComment(null, draft)) draft = '';
  }
</script>

<section class:embedded class="discussion-shell" id="comments">
  <div class="composer-card">
    <CommentComposer bind:value={draft} {signedIn} {onSignIn} onSubmit={() => void submit()} />
  </div>

  <div class="stack">
    {#each sorted as comment (comment.id)}
      <DiscussionComment
        {comment}
        {space}
        {names}
        {highlighted}
        {embedded}
        {signedIn}
        {onSignIn}
        {onVote}
        onReply={(parent, body) => onComment(parent, body)}
        {onModeration}
      />
    {/each}
  </div>
</section>

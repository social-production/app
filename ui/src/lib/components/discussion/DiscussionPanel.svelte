<script lang="ts">
  import {
    sortComments,
    type CommentSort,
    type DiscussionComment as Comment,
    type VoteDirection,
  } from '#lib/domain/discussions.ts';
  import type { Moderation } from '#lib/domain/moderation.ts';

  import BackButton from '#lib/components/BackButton.svelte';

  import type { CommentMentions } from './commentMentions.ts';
  import CommentComposer from './CommentComposer.svelte';
  import DiscussionComment from './DiscussionComment.svelte';

  import '#lib/components/styles/discussion.css';

  /**
   * web's DiscussionPanel: the comment composer and the comments, top-level ones in the chosen
   * order (ADR 0070). On phones the composer is a floating bar with a back button in the
   * nav's place (ADR 0100).
   */
  interface Props {
    comments: Comment[];
    /** The thread's or post's listing id. */
    space: string;
    names: Record<string, string>;
    sortMode: CommentSort;
    highlighted: string | null;
    embedded?: boolean;
    /** Posts (ADR 0100): Twitter-style comments, replies at the same level. Threads nest. */
    flat?: boolean;
    signedIn: boolean;
    onSignIn: (message: string) => void;
    onVote: (id: string, direction: VoteDirection) => Promise<void>;
    /** Writes a comment (`parent` null) or a reply; resolves true once written. */
    onComment: (parent: string | null, body: string) => Promise<boolean>;
    onModeration: (id: string, next: Moderation) => void;
    mentions?: CommentMentions | null;
  }

  let {
    comments,
    space,
    names,
    sortMode,
    highlighted,
    embedded = false,
    flat = false,
    signedIn,
    onSignIn,
    onVote,
    onComment,
    onModeration,
    mentions = null,
  }: Props = $props();

  let draft = $state('');
  const sorted = $derived(sortComments(comments, sortMode));

  async function submit(): Promise<void> {
    if (await onComment(null, draft)) draft = '';
  }
</script>

<section class:embedded class="discussion-shell" id="comments">
  <div class="composer-card">
    <BackButton />
    <CommentComposer
      bind:value={draft}
      {signedIn}
      {onSignIn}
      mentionSearch={mentions?.search}
      onSubmit={() => void submit()}
    />
  </div>

  <div class="stack">
    {#each sorted as comment (comment.id)}
      <DiscussionComment
        {comment}
        {space}
        {names}
        {highlighted}
        {embedded}
        {flat}
        {signedIn}
        {onSignIn}
        {onVote}
        onReply={(parent, body) => onComment(parent, body)}
        {onModeration}
        {mentions}
      />
    {/each}
  </div>
</section>

<script lang="ts">
  import { t } from '#lib/i18n/index.ts';
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
   * nav's place (ADR 0100). It is the page's only composer: Reply points it at a comment, and
   * a bar above it says whom you answer until you send or press × (Profiles and Comments
   * mockups §4).
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
    /** Posts: a comment opens its own view (its replies are not listed under it). */
    onOpen?: ((id: string) => void) | undefined;
    /** An opened comment: what the composer answers when no Reply was pressed. */
    parent?: { id: string; author: string } | null;
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
    onOpen,
    parent = null,
  }: Props = $props();

  let draft = $state('');
  let replying = $state<{ id: string; author: string } | null>(null);
  // Names may arrive after Reply is pressed.
  const answering = $derived(replying ?? parent);
  const replyName = $derived(answering ? (names[answering.author] ?? 'Unnamed') : '');
  let focus = $state(0);
  const sorted = $derived(sortComments(comments, sortMode));

  function startReply(id: string, author: string): void {
    if (!signedIn) return onSignIn('Sign in to reply.');
    replying = { id, author };
    focus++;
    // The comment being answered goes to the top of the screen, once the keyboard is up
    // (owner, 2026-11-24).
    setTimeout(
      () =>
        document
          .getElementById(`comment-${id}`)
          ?.scrollIntoView({ block: 'start', behavior: 'smooth' }),
      350,
    );
  }

  async function submit(): Promise<void> {
    const to = answering?.id ?? null;
    const body = draft;
    draft = '';
    replying = null;
    if (!(await onComment(to, body))) draft = body;
  }
</script>

<section class:embedded class="discussion-shell" id="comments">
  <div class="composer-card" class:replying={!!replying}>
    {#if replying}
      <div class="reply-bar">
        <span>{t('Replying to')} <b>{replyName}</b></span>
        <button type="button" aria-label={t('Cancel reply')} onclick={() => (replying = null)}
          >×</button
        >
      </div>
    {/if}
    <BackButton />
    <CommentComposer
      bind:value={draft}
      placeholder={answering ? t('Reply to {name}', { name: replyName }) : t('Add a comment')}
      submitLabel={answering ? t('Post reply') : t('Post comment')}
      {focus}
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
        replyTarget={replying?.id ?? null}
        {onVote}
        onReply={startReply}
        {onModeration}
        {mentions}
        {onOpen}
      />
    {/each}
  </div>
</section>

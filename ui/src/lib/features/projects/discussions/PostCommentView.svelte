<script lang="ts">
  import { t } from '#lib/i18n/index.ts';
  import type { CommentMentions } from '#lib/components/discussion/commentMentions.ts';
  import DiscussionComment from '#lib/components/discussion/DiscussionComment.svelte';
  import DiscussionPanel from '#lib/components/discussion/DiscussionPanel.svelte';
  import { commentPath } from '#lib/domain/discussions.ts';

  import type { DiscussionState } from './discussionState.svelte.ts';

  /**
   * A post's comment opened as its own view (owner's screenshots, 2026-11-24, as on Twitter):
   * the comments it answers above it on a line from the post, the comment itself, then its
   * replies as the list, and the composer answering it. A reply opens the same way.
   */
  interface Props {
    state: DiscussionState;
    space: string;
    focus: string;
    highlighted: string | null;
    signedIn: boolean;
    mentions: CommentMentions | null;
    onSignIn: (message: string) => void;
    onOpen: (id: string) => void;
  }

  let { state, space, focus, highlighted, signedIn, mentions, onSignIn, onOpen }: Props = $props();

  // The composer already answers the opened comment; its Reply just brings the composer up.
  const focusComposer = (): void =>
    document.querySelector<HTMLElement>('.discussion-shell .composer-input')?.focus();

  const path = $derived(commentPath(state.comments, focus));
  const opened = $derived(path?.at(-1) ?? null);
  const above = $derived(path?.slice(0, -1) ?? []);
  const common = $derived({
    space,
    names: state.names,
    highlighted,
    embedded: true,
    flat: true,
    onVote: (c: string, d: -1 | 0 | 1) => state.voteComment(c, d),
    onModeration: (c: string, next: Parameters<typeof state.setCommentModeration>[1]) =>
      state.setCommentModeration(c, next),
    mentions,
  });
</script>

{#if !opened}
  <p class="tab-loading">{state.comments.length ? t('This comment is gone.') : t('Loading…')}</p>
{:else}
  <div class="post-comment-chain">
    {#each above as comment (comment.id)}
      <DiscussionComment
        {comment}
        {...common}
        hideReplies
        onReply={() => onOpen(comment.id)}
        {onOpen}
      />
    {/each}
    <div class="post-comment-opened">
      <DiscussionComment comment={opened} {...common} hideReplies onReply={focusComposer} />
    </div>
  </div>
  <h2 class="post-comment-replies">{t('Replies')}</h2>
  <DiscussionPanel
    comments={opened.replies}
    {space}
    names={state.names}
    sortMode={state.sortMode}
    {highlighted}
    embedded
    flat
    {signedIn}
    {mentions}
    {onSignIn}
    onVote={(c, d) => state.voteComment(c, d)}
    onComment={(parent, body) => state.comment(parent, body)}
    onModeration={(c, next) => state.setCommentModeration(c, next)}
    {onOpen}
    parent={{ id: opened.id, author: opened.author }}
  />
{/if}

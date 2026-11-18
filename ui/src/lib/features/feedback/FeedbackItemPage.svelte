<script lang="ts">
  import { t } from '#lib/i18n/index.ts';
  import LoadingLine from '#lib/components/loading/LoadingLine.svelte';
  import SkeletonDetail from '#lib/components/loading/SkeletonDetail.svelte';
  import BackButton from '#lib/components/BackButton.svelte';
  import CommentComposer from '#lib/components/discussion/CommentComposer.svelte';
  import FeedbackVoteStrip from '#lib/components/feedback/FeedbackVoteStrip.svelte';
  import AvatarBadge from '#lib/components/icons/AvatarBadge.svelte';
  import ToolbarIcon from '#lib/components/icons/ToolbarIcon.svelte';
  import { approvalPercent } from '#lib/domain/feedback.ts';
  import { relativeCompact } from '#lib/domain/time.ts';
  import { requireViewer } from '#lib/stores/authNotice.svelte.ts';
  import { connection } from '#lib/stores/connection.svelte.ts';

  import { resolve } from '$app/paths';

  import { FeedbackItemState } from './feedbackItemState.svelte.ts';

  import '#lib/components/styles/discussion.css';
  import '#lib/components/styles/feedback.css';

  /**
   * A feedback item on its own page, built like a post (Plan, Phase and Feedback Screens
   * §7): the item with its vote, comment count and Share as soft pills, then flat comments
   * with "Replying to …" and likes, and the floating comment bar with the back button.
   */
  let { id }: { id: string } = $props();

  // svelte-ignore state_referenced_locally
  const fb = new FeedbackItemState(id);
  const signedIn = $derived(connection.current.status === 'connected');
  let draft = $state('');
  let replyTo = $state<string | null>(null);
  let shared = $state(false);

  $effect(() => {
    void fb.load();
  });

  const name = (agent: string): string => fb.names[agent] ?? '…';
  const replyName = $derived(
    replyTo ? name(fb.comments.find((c) => c.id === replyTo)?.author ?? '') : '',
  );

  async function send(): Promise<void> {
    if (await fb.comment(draft, replyTo)) {
      draft = '';
      replyTo = null;
    }
  }

  function answer(commentId: string): void {
    if (!requireViewer('Sign in to reply.')) return;
    replyTo = commentId;
    document.querySelector<HTMLTextAreaElement>('.discussion-shell textarea')?.focus();
  }

  async function share(): Promise<void> {
    await navigator.clipboard?.writeText(location.href).catch(() => undefined);
    shared = true;
    setTimeout(() => (shared = false), 2000);
  }
</script>

<div class="feedback-item-page">
  {#if fb.item}
    {@const item = fb.item}
    <article class="feedback-item">
      <div class="feedback-item-head">
        <span class={`kind ${item.kind}`} aria-hidden="true">
          <ToolbarIcon name={item.kind === 'bug' ? 'bug' : 'lightbulb'} />
        </span>
        <span class="feedback-item-who">
          <b>{item.kind === 'bug' ? t('Bug') : t('Suggestion')}</b>
          <small>
            {t('by')}
            <a href={resolve('/profile/[agent]', { agent: item.author })}>{name(item.author)}</a>
            · {relativeCompact(item.createdAt)}
          </small>
        </span>
      </div>
      <h1>{item.title}</h1>
      <p class="feedback-item-body">{item.description}</p>
      <div class="feedback-item-pills">
        <FeedbackVoteStrip
          upvotes={item.upvotes}
          downvotes={item.downvotes}
          approval={approvalPercent(item)}
          mine={item.myVote}
          onVote={(vote) => void fb.vote(vote)}
        />
        <span class="feedback-pill" aria-label={`${item.comments} comments`}>
          <ToolbarIcon name="message" />{item.comments}
        </span>
        <button
          class="feedback-pill"
          type="button"
          aria-label={t('Share')}
          onclick={() => void share()}
        >
          <svg aria-hidden="true" viewBox="0 0 24 24"
            ><path d="M12 15V4M8 8l4-4 4 4M5 13v6h14v-6" /></svg
          >{shared ? t('Copied') : ''}
        </button>
      </div>
    </article>

    <section class="discussion-shell feedback-comments" id="comments">
      <div class="composer-card">
        <BackButton />
        <CommentComposer
          bind:value={draft}
          placeholder={replyTo ? `Reply to ${replyName}` : t('Add a comment')}
          {signedIn}
          onSignIn={(message) => requireViewer(message)}
          onSubmit={() => void send()}
        />
      </div>
      {#if replyTo}
        <p class="replying-note">
          {t('Replying to')} <b>{replyName}</b>
          <button type="button" onclick={() => (replyTo = null)}>{t('Cancel')}</button>
        </p>
      {/if}
      {#each fb.comments as comment (comment.id)}
        {@const parent = fb.comments.find((c) => c.id === comment.replyTo)}
        <article class="feedback-comment">
          <AvatarBadge size="sm" name={name(comment.author)} />
          <div class="feedback-comment-copy">
            <p class="feedback-comment-head">
              <a href={resolve('/profile/[agent]', { agent: comment.author })}
                >{name(comment.author)}</a
              >
              · {relativeCompact(comment.createdAt)}
            </p>
            {#if parent}<p class="replying-to">Replying to {name(parent.author)}</p>{/if}
            <p class="feedback-comment-body">{comment.body}</p>
            <div class="feedback-comment-actions">
              <button type="button" aria-label={t('Reply')} onclick={() => answer(comment.id)}>
                <ToolbarIcon name="message" />
              </button>
              <button
                type="button"
                class:liked={comment.liked}
                aria-pressed={comment.liked}
                aria-label={comment.liked ? t('Unlike') : t('Like')}
                onclick={() => void fb.like(comment)}
              >
                <svg aria-hidden="true" viewBox="0 0 24 24"
                  ><path
                    d="M12 20s-7-4.4-7-10a4 4 0 0 1 7-2.6A4 4 0 0 1 19 10c0 5.6-7 10-7 10Z"
                  /></svg
                >{comment.likes || ''}
              </button>
            </div>
          </div>
        </article>
      {:else}
        <p class="feedback-note">{t('No comments yet.')}</p>
      {/each}
    </section>
  {:else}
    {#if fb.loaded}
      <p class="feedback-note">{fb.error}</p>
    {:else}
      <LoadingLine /><SkeletonDetail />
    {/if}
  {/if}
  {#if fb.item && fb.error}<p class="feedback-note warning" role="alert">{fb.error}</p>{/if}
</div>

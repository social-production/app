<script lang="ts">
  import { t } from '#lib/i18n/index.ts';
  import { shortDate, type BotReview } from '#lib/domain/botReview.ts';

  /** web's ProfilePage `.bot-review-card`: an open review with its tally and votes, or a
   * paused account's verdict and re-review (ADR 0074). */
  interface Props {
    review: BotReview;
    own: boolean;
    canVote: boolean;
    pending: boolean;
    message: string;
    onVote: (vote: 'bot' | 'real') => void;
    onReReview: () => void;
  }

  let { review, own, canVote, pending, message, onVote, onReReview }: Props = $props();

  const canReReview = $derived(
    own && review.reReviewFrom !== null && review.reReviewFrom <= Date.now(),
  );
</script>

<section class="bot-review-card" class:upheld={review.status === 'upheld'} aria-live="polite">
  {#if review.status === 'open'}
    <div class="bot-review-copy">
      <strong>{review.isAppeal ? t('Re-review open') : t('Bot review open')}</strong>
      <p>
        {own
          ? t(
              'People are voting on whether this account belongs to a real person. You can keep using the site meanwhile.',
            )
          : t('People are voting on whether this account belongs to a real person.')}
      </p>
      <p class="bot-review-tally">
        {review.realVotes} real · {review.botVotes} bot · needs {review.votesRequired} votes, 66% bot
        to pass
        {#if review.closesAt}
          · closes {shortDate(review.closesAt)}
        {/if}
      </p>
    </div>
    {#if canVote}
      {#if review.myVote}
        <p class="bot-review-tally">
          You voted {review.myVote === 'bot' ? t('Bot') : t('Real person')}. Tap the other to change
          it.
        </p>
      {/if}
      <div class="bot-review-actions">
        <button
          type="button"
          class:chosen={review.myVote === 'real'}
          aria-pressed={review.myVote === 'real'}
          disabled={pending}
          onclick={() => onVote('real')}>{t('Real person')}</button
        >
        <button
          type="button"
          class="bot"
          class:chosen={review.myVote === 'bot'}
          aria-pressed={review.myVote === 'bot'}
          disabled={pending}
          onclick={() => onVote('bot')}>{t('Bot')}</button
        >
      </div>
    {/if}
  {:else}
    <div class="bot-review-copy">
      <strong>{t('Found to be a bot')}</strong>
      <p>{t('A community review decided this account is a bot, so it is read-only.')}</p>
      {#if canReReview}
        <button type="button" class="re-review" disabled={pending} onclick={onReReview}>
          {t('Ask for a re-review')}
        </button>
      {:else if own && review.reReviewFrom !== null}
        <p class="bot-review-tally">
          You can ask for one re-review from {shortDate(review.reReviewFrom)}.
        </p>
      {/if}
    </div>
  {/if}
  {#if message}
    <p class="bot-review-error" role="alert">{message}</p>
  {/if}
</section>

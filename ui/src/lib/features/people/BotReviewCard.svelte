<script lang="ts">
  import { t } from '#lib/i18n/index.ts';
  import { APPROVAL_PERCENT } from '#lib/domain/governance.ts';
  import { shortDate, type BotReview } from '#lib/domain/botReview.ts';

  /** web's ProfilePage `.bot-review-card`: an open review with its tally and votes, or a
   * paused account's verdict and re-review (ADR 0074). The tally is the vote sheets' thin bar
   * towards 66% bot, and Real person | Bot one joined capsule, as every vote control is. */
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
  const cast = $derived(review.botVotes + review.realVotes);
  const botShare = $derived(cast > 0 ? Math.round((review.botVotes / cast) * 100) : 0);
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
    </div>
    <div class="bot-review-tally" aria-label={t('Votes')}>
      <div class="bot-review-bar">
        <i style={`width: ${botShare}%`}></i><span style={`left: ${APPROVAL_PERCENT}%`}></span>
      </div>
      <div class="bot-review-line">
        <span
          ><b>{t('{bot} bot · {real} real', { bot: review.botVotes, real: review.realVotes })}</b>
          · {t('{n} of {total} votes', { n: cast, total: review.votesRequired })}</span
        >
        <span>{t('needs {percent}% bot', { percent: APPROVAL_PERCENT })}</span>
      </div>
      {#if review.closesAt}
        <p class="bot-review-note">{t('Closes {date}', { date: shortDate(review.closesAt) })}</p>
      {/if}
    </div>
    {#if canVote}
      <div class="bot-review-actions" role="group" aria-label={t('Your vote')}>
        <button
          type="button"
          class:chosen={review.myVote === 'real'}
          aria-pressed={review.myVote === 'real'}
          disabled={pending}
          onclick={() => onVote('real')}
          >{review.myVote === 'real' ? '✓ ' : ''}{t('Real person')}</button
        >
        <button
          type="button"
          class="bot"
          class:chosen={review.myVote === 'bot'}
          aria-pressed={review.myVote === 'bot'}
          disabled={pending}
          onclick={() => onVote('bot')}>{review.myVote === 'bot' ? '✓ ' : ''}{t('Bot')}</button
        >
      </div>
      {#if review.myVote}<p class="bot-review-note">
          {t('Tap the other to change your vote.')}
        </p>{/if}
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
        <p class="bot-review-note">
          {t('You can ask for one re-review from {date}.', {
            date: shortDate(review.reReviewFrom),
          })}
        </p>
      {/if}
    </div>
  {/if}
  {#if message}
    <p class="bot-review-error" role="alert">{message}</p>
  {/if}
</section>

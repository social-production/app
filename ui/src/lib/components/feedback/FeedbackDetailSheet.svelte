<script lang="ts">
  import OverlaySheet from '#lib/components/overlay/OverlaySheet.svelte';
  import { approvalPercent, type FeedbackItem, type FeedbackVote } from '#lib/domain/feedback.ts';
  import { relativeCompact } from '#lib/domain/time.ts';

  import FeedbackVoteStrip from './FeedbackVoteStrip.svelte';

  import '#lib/components/styles/feedback.css';

  /** web's FeedbackDetailSheet: kind, byline, title, description and the docked vote strip. */
  interface Props {
    item: FeedbackItem | null;
    authorName: string;
    pendingVote: boolean;
    onVote: (vote: FeedbackVote) => void;
    onClose: () => void;
  }

  let { item, authorName, pendingVote, onVote, onClose }: Props = $props();
</script>

<OverlaySheet
  open={item !== null}
  title={item?.kind === 'bug' ? 'Bug' : 'Suggestion'}
  labelledById="feedback-detail-sheet"
  rootClass="feedback-sheet"
  wide
  {onClose}
>
  {#snippet subtitle()}
    {#if item}
      <span class="sheet-byline">
        <span>by {authorName}</span>
        <span aria-hidden="true">·</span>
        <span>{relativeCompact(item.createdAt)}</span>
      </span>
    {/if}
  {/snippet}
  {#if item}
    <div class="detail-stack">
      <h3 class="detail-title">{item.title}</h3>
      <p class="description">{item.description}</p>
    </div>
  {/if}
  {#snippet footer()}
    <div class="vote-dock">
      {#if item}
        <FeedbackVoteStrip
          upvotes={item.upvotes}
          downvotes={item.downvotes}
          approval={approvalPercent(item)}
          mine={item.myVote}
          docked
          disabled={pendingVote}
          {onVote}
        />
      {/if}
    </div>
  {/snippet}
</OverlaySheet>

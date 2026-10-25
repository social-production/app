<script lang="ts">
  import FeedSurface from '#lib/components/cards/FeedSurface.svelte';
  import ToolbarIcon from '#lib/components/icons/ToolbarIcon.svelte';
  import { approvalPercent, type FeedbackItem, type FeedbackVote } from '#lib/domain/feedback.ts';

  import FeedbackVoteStrip from './FeedbackVoteStrip.svelte';

  import '#lib/components/styles/feedback.css';

  /** web's FeedbackCard: kind icon, title, vote strip; the row opens the detail sheet. */
  interface Props {
    item: FeedbackItem;
    pendingVote: boolean;
    onOpen: () => void;
    onVote: (vote: FeedbackVote) => void;
  }

  let { item, pendingVote, onOpen, onVote }: Props = $props();

  function click(event: MouseEvent): void {
    if ((event.target as HTMLElement).closest('button, a')) return;
    onOpen();
  }

  function keydown(event: KeyboardEvent): void {
    if (event.key !== 'Enter' && event.key !== ' ') return;
    event.preventDefault();
    onOpen();
  }
</script>

<FeedSurface compact accent={null}>
  <div
    aria-label={`Open ${item.kind}: ${item.title}`}
    class="feedback-card"
    onclick={click}
    onkeydown={keydown}
    role="button"
    tabindex="0"
  >
    <span class={`kind ${item.kind}`} title={item.kind} aria-hidden="true">
      <ToolbarIcon name={item.kind === 'bug' ? 'bug' : 'lightbulb'} />
    </span>
    <span class="title-wrap">
      <span class="feedback-title">{item.title}</span>
    </span>
    <div class="votes">
      <FeedbackVoteStrip
        upvotes={item.upvotes}
        downvotes={item.downvotes}
        approval={approvalPercent(item)}
        mine={item.myVote}
        disabled={pendingVote}
        {onVote}
      />
    </div>
  </div>
</FeedSurface>

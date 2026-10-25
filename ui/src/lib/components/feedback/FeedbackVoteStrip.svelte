<script lang="ts">
  import type { FeedbackVote } from '#lib/domain/feedback.ts';

  /** web's FeedbackVoteStrip: ▲ support · approval % · ▼ oppose; your vote again clears it. */
  interface Props {
    upvotes: number;
    downvotes: number;
    approval: number;
    mine: FeedbackVote;
    docked?: boolean;
    disabled?: boolean;
    onVote: (vote: FeedbackVote) => void;
  }

  let {
    upvotes,
    downvotes,
    approval,
    mine,
    docked = false,
    disabled = false,
    onVote,
  }: Props = $props();

  const percent = $derived(`${Math.round(approval)}%`);
  const tooltip = $derived(`${upvotes} support · ${downvotes} oppose · ${percent}`);

  function vote(value: 'up' | 'down', event: MouseEvent): void {
    event.stopPropagation();
    onVote(mine === value ? null : value);
  }
</script>

<span class="feedback-strip">
  <span class="vote-strip signal-strip" class:disabled class:docked title={tooltip}>
    <button
      aria-label={`Support · ${upvotes}`}
      aria-pressed={mine === 'up'}
      class="signal-button vote-button"
      class:active-support={mine === 'up'}
      {disabled}
      type="button"
      onclick={(e) => vote('up', e)}
    >
      ▲
      {#if docked}
        <span class="signal-label">Support</span>
        <span class="signal-count">{upvotes}</span>
      {/if}
    </button>
    <span class="signal-percent" aria-label={tooltip}>{percent}</span>
    <button
      aria-label={`Oppose · ${downvotes}`}
      aria-pressed={mine === 'down'}
      class="signal-button vote-button"
      class:active-oppose={mine === 'down'}
      {disabled}
      type="button"
      onclick={(e) => vote('down', e)}
    >
      {#if docked}
        <span class="signal-label">Oppose</span>
        <span class="signal-count">{downvotes}</span>
      {/if}
      ▼
    </button>
  </span>
</span>

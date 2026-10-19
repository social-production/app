<script lang="ts">
  import { applyVote, type VoteDirection } from '#lib/domain/threads.ts';

  /**
   * web's VoteStrip in votes mode: ▲ count ▼. Clicking your current vote takes it back. The
   * count moves at once and rolls back if the write fails.
   */
  interface Props {
    count: number;
    active: VoteDirection;
    corner?: boolean;
    disabled?: boolean;
    onVote: (next: VoteDirection) => Promise<void>;
  }

  let { count, active, corner = false, disabled = false, onVote }: Props = $props();

  let local = $state<{ myVote: VoteDirection; voteCount: number } | null>(null);
  const shown = $derived(local ?? { myVote: active, voteCount: count });

  // Fresh numbers from a reload replace the local ones.
  $effect(() => {
    void count;
    void active;
    local = null;
  });

  async function click(button: 1 | -1, event: MouseEvent): Promise<void> {
    event.stopPropagation();
    if (disabled) return;
    const before = shown;
    const target: VoteDirection = before.myVote === button ? 0 : button;
    local = applyVote(before.myVote, before.voteCount, target);
    try {
      await onVote(target);
    } catch {
      local = before;
    }
  }

  import '#lib/components/styles/cards.css';
</script>

<div class="vote-strip" class:corner>
  <button
    aria-label="Vote up"
    aria-pressed={shown.myVote === 1}
    class:active-up={shown.myVote === 1}
    class="vote-button"
    {disabled}
    type="button"
    onclick={(e) => click(1, e)}
  >
    ▲
  </button>
  <span
    class:active-up={shown.myVote === 1}
    class:active-down={shown.myVote === -1}
    class="vote-count"
  >
    {shown.voteCount}
  </span>
  <button
    aria-label="Vote down"
    aria-pressed={shown.myVote === -1}
    class:active-down={shown.myVote === -1}
    class="vote-button"
    {disabled}
    type="button"
    onclick={(e) => click(-1, e)}
  >
    ▼
  </button>
</div>

<script lang="ts" module>
  export type BoardAsk = 'volunteer' | 'withdraw' | 'step-down';
</script>

<script lang="ts">
  import OverlaySheet from '#lib/components/overlay/OverlaySheet.svelte';

  /**
   * One question before volunteering, withdrawing or stepping down (Platform and Moderators
   * mockups §3), with this week's real numbers, so nobody joins or leaves by accident.
   */
  interface Props {
    ask: BoardAsk | null;
    quorum: number;
    working: boolean;
    onConfirm: (ask: BoardAsk) => void;
    onClose: () => void;
  }

  let { ask, quorum, working, onConfirm, onClose }: Props = $props();

  const copy = $derived.by(() => {
    if (ask === 'withdraw') {
      return {
        title: 'Withdraw your candidacy?',
        body: ["You'll leave the candidates list. You can volunteer again later."],
        yes: 'Withdraw',
      };
    }
    if (ask === 'step-down') {
      return {
        title: 'Step down as moderator?',
        body: ["You'll leave the board straight away. To come back, volunteer again."],
        yes: 'Step down',
      };
    }
    const seat =
      quorum === 0
        ? "Nobody voted last week, so you'll be seated straight away. Once people are active, your seat needs their votes."
        : `You'll be listed as a candidate. Once ${quorum} people have voted and at least 66% say yes, you're seated.`;
    return {
      title: 'Volunteer as moderator?',
      body: [seat, 'You can step down any time.'],
      yes: 'Volunteer',
    };
  });
</script>

<OverlaySheet
  open={ask !== null}
  title={copy.title}
  labelledById="board-confirm-title"
  rootClass="platform-board"
  elevated
  {onClose}
>
  <div class="board-confirm">
    {#each copy.body as line, i (i)}<p class:soft={i > 0}>{line}</p>{/each}
  </div>
  {#snippet footer()}
    <div class="sheet-actions">
      <button
        class="sheet-submit"
        type="button"
        disabled={working}
        onclick={() => ask && onConfirm(ask)}>{working ? 'Working…' : copy.yes}</button
      >
    </div>
  {/snippet}
</OverlaySheet>

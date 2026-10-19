<script lang="ts">
  import type { VoteChoice } from '#lib/domain/governance.ts';

  /** web's VoteChoiceBar. Votes are final here (ADR 0033), so a chosen bar is disabled. */
  let { mine, onChoose }: { mine: VoteChoice | null; onChoose: (choice: VoteChoice) => void } =
    $props();
</script>

<div class="choice-bar" class:chosen={!!mine}>
  <button
    class="choice yes"
    class:selected={mine === 'Yes'}
    type="button"
    aria-pressed={mine === 'Yes'}
    disabled={!!mine}
    onclick={() => onChoose('Yes')}>Yes</button
  >
  <button
    class="choice no"
    class:selected={mine === 'No'}
    type="button"
    aria-pressed={mine === 'No'}
    disabled={!!mine}
    onclick={() => onChoose('No')}>No</button
  >
</div>

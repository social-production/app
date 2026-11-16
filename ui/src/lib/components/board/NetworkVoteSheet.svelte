<script lang="ts">
  import OverlaySheet from '#lib/components/overlay/OverlaySheet.svelte';

  /**
   * What a network vote is (Platform and Moderators mockups §1, §4; ADR 0095): on a project
   * tagged #platform, any Full member can vote, not only its members, and the votes needed
   * are sized from the people active last week.
   */
  let {
    open,
    weeklyActive = null,
    onClose,
  }: { open: boolean; weeklyActive?: number | null; onClose: () => void } = $props();

  const sized = $derived(
    !weeklyActive
      ? 'the people active last week'
      : `the ${weeklyActive.toLocaleString()} ${weeklyActive === 1 ? 'person' : 'people'} active last week`,
  );
</script>

<OverlaySheet
  {open}
  title="Network votes"
  labelledById="network-vote-title"
  rootClass="platform-board"
  elevated
  {onClose}
>
  <div class="board-confirm">
    <p>
      A project tagged #platform changes the whole network, so any Full member can vote on its
      plans, phase moves, edits and updates, not only its members. Nobody has to vote.
    </p>
    <p class="soft">
      The votes a decision needs are sized from {sized}, and it passes with 66% yes, as every vote
      here does.
    </p>
  </div>
</OverlaySheet>

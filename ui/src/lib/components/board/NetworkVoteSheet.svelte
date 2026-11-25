<script lang="ts">
  import { t } from '#lib/i18n/index.ts';
  import OverlaySheet from '#lib/components/overlay/OverlaySheet.svelte';

  /**
   * What a network vote is (Platform and Moderators mockups §1, §4; ADR 0095): on a project
   * tagged #platform, anyone can vote, not only its members, and the votes needed
   * are sized from the people active in the last 7 days.
   */
  let {
    open,
    weeklyActive = null,
    onClose,
  }: { open: boolean; weeklyActive?: number | null; onClose: () => void } = $props();

  const sized = $derived(
    !weeklyActive
      ? t('the people active in the last 7 days')
      : `the ${weeklyActive.toLocaleString()} ${weeklyActive === 1 ? 'person' : 'people'} active in the last 7 days`,
  );
</script>

<OverlaySheet
  {open}
  title={t('Network votes')}
  labelledById="network-vote-title"
  rootClass="platform-board"
  elevated
  {onClose}
>
  <div class="board-confirm">
    <p>
      {t(
        'A project tagged #platform changes the whole network, so anyone can vote on its plans, phase moves, edits and updates, not only its members. Nobody has to vote.',
      )}
    </p>
    <p class="soft">
      The votes a decision needs are sized from {sized}, and it passes with 66% yes, as every vote
      here does.
    </p>
  </div>
</OverlaySheet>

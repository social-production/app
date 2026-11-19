<script lang="ts">
  import { t } from '#lib/i18n/index.ts';

  import SkeletonRows from './SkeletonRows.svelte';

  import '#lib/components/styles/ux/loading.css';

  /** While the first connection is made (owner, 2026-11-19: "just use the same one we use for
   * loading"): the feed's breathing cards under one quiet line saying what is happening, as
   * app.html's screen before the code arrives; after 8 s a note that a slow network can take a
   * minute. */
  let slow = $state(false);
  $effect(() => {
    const timer = setTimeout(() => (slow = true), 8000);
    return () => clearTimeout(timer);
  });
</script>

<div class="opening-screen" role="status">
  <p>{t('Connecting to the network')}</p>
  {#if slow}<p>{t('Still connecting. This can take a minute on a slow network.')}</p>{/if}
  <SkeletonRows count={4} />
</div>

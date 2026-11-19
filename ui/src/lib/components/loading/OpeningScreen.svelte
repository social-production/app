<script lang="ts">
  import { t } from '#lib/i18n/index.ts';
  import brandIcon from '#lib/assets/brand/app-icon-no-background-full-white.png';

  import '#lib/components/styles/ux/loading.css';

  /** Owner's fix 3c (option A): the logo in its green tile, breathing, the name and one line
   * saying what is happening; after 8 s a note that a slow network can take a minute. */
  let { message = 'Connecting to the network' }: { message?: string } = $props();

  let slow = $state(false);
  $effect(() => {
    const timer = setTimeout(() => (slow = true), 8000);
    return () => clearTimeout(timer);
  });
</script>

<div class="opening-screen" role="status">
  <span class="opening-logo"><img alt="" src={brandIcon} /></span>
  <strong>{t('Social Production')}</strong>
  <span class="opening-dots" aria-hidden="true"><i></i><i></i><i></i></span>
  <p>{t(message)}…</p>
  {#if slow}<p class="opening-slow">
      {t('Still connecting. This can take a minute on a slow network.')}
    </p>{/if}
</div>

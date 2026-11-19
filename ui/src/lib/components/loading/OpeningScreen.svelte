<script lang="ts">
  import { t } from '#lib/i18n/index.ts';

  import '#lib/components/styles/ux/loading.css';

  /** Owner's fix 3c, quietened on 2026-11-19: as app.html's opening screen, the logo mark, the
   * name, a thin moving line and what is happening; after 8 s a note that a slow network can
   * take a minute. */
  let { message = 'Connecting to the network' }: { message?: string } = $props();

  let slow = $state(false);
  $effect(() => {
    const timer = setTimeout(() => (slow = true), 8000);
    return () => clearTimeout(timer);
  });
</script>

<div class="opening-screen" role="status">
  <span class="opening-logo" aria-hidden="true"></span>
  <strong>{t('Social Production')}</strong>
  <span class="opening-track" aria-hidden="true"></span>
  <p>{message}</p>
  {#if slow}<p class="opening-slow">
      {t('Still connecting. This can take a minute on a slow network.')}
    </p>{/if}
</div>

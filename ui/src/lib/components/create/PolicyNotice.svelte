<script lang="ts">
  import { t } from '#lib/i18n/index.ts';
  import OverlaySheet from '#lib/components/overlay/OverlaySheet.svelte';
  import {
    DIRECT_USE_CORE,
    DIRECT_USE_CREATE,
    DIRECT_USE_EVENT_CREATE,
    DIRECT_USE_REQUEST,
  } from '#lib/domain/policy.ts';

  /**
   * web's DirectUsePolicyNotice (`context="event"` on the create-event page, `variant=
   * "request"` on the help-request one), as one chip that opens the full rule in a sheet
   * (Polish Pass mockups, 2026-11-16).
   */
  let { event = false, request = false }: { event?: boolean; request?: boolean } = $props();

  let open = $state(false);
</script>

<aside class="policy-notice policy-chip" aria-label={t('Direct-use policy')}>
  <svg aria-hidden="true" viewBox="0 0 24 24"
    ><path
      d="M8 13V6a1.5 1.5 0 0 1 3 0v6M11 11V4.5a1.5 1.5 0 0 1 3 0V11M14 11V6a1.5 1.5 0 0 1 3 0v8a6 6 0 0 1-6 6h-1a6 6 0 0 1-5-3l-2-4a1.5 1.5 0 0 1 2.6-1.5L8 13"
    /></svg
  >
  <span>{t('Direct use only: nothing here is sold')}</span>
  <button type="button" aria-haspopup="dialog" onclick={() => (open = true)}>{t('Read')}</button>
</aside>

<OverlaySheet
  {open}
  title={t('Direct use only')}
  labelledById="direct-use-rule"
  rootClass="create-sheet policy-sheet"
  elevated
  onClose={() => (open = false)}
>
  <div class="policy-text">
    <p>{event ? DIRECT_USE_EVENT_CREATE : request ? DIRECT_USE_REQUEST : DIRECT_USE_CREATE}</p>
    <p>{DIRECT_USE_CORE}</p>
  </div>
</OverlaySheet>

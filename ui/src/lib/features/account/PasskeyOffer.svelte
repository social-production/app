<script lang="ts">
  import { t } from '#lib/i18n/index.ts';
  import { connection } from '#lib/stores/connection.svelte.ts';
  import { viewer } from '#lib/stores/viewer.svelte.ts';
  import { addPasskey, devicePasskeysAvailable } from '#lib/services/account.ts';
  import { explain } from '#lib/services/errors.ts';
  import OverlaySheet from '#lib/components/overlay/OverlaySheet.svelte';

  import '#lib/components/styles/ux/passkeysheet.css';

  /** The passkey offer after signing up or signing in (mockup option C, owner 2026-11-19):
   * a sheet over the feed, once per device; × or the scrim means Not now. */
  const KEY = 'sp-passkey-offer';
  let visible = $state(false);
  let busy = $state(false);
  let error = $state<string>();

  $effect(() => {
    if (!offerDismissed())
      void devicePasskeysAvailable().then((available) => (visible = available));
  });

  function offerDismissed(): boolean {
    try {
      return localStorage.getItem(KEY) !== null;
    } catch {
      return false; // storage blocked: offer again
    }
  }

  function close(value: 'saved' | 'dismissed'): void {
    visible = false;
    try {
      localStorage.setItem(KEY, value);
    } catch {
      // storage blocked: the offer just comes back next time
    }
  }

  async function save(): Promise<void> {
    busy = true;
    error = undefined;
    try {
      await addPasskey(viewer.name || 'Social Production');
      close('saved');
    } catch (e) {
      error = explain(e);
    } finally {
      busy = false;
    }
  }
</script>

<OverlaySheet
  open={visible && connection.current.status === 'connected'}
  title={t('Next time, just use your fingerprint or face')}
  labelledById="passkey-offer-title"
  rootClass="passkey-sheet"
  onClose={() => close('dismissed')}
>
  {#snippet kicker()}
    <span class="passkey-badge" aria-hidden="true">
      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8">
        <path d="M12 11c0 3.5-1 6.5-2.5 8.5" />
        <path d="M8.5 10.5a3.5 3.5 0 0 1 7 0c0 1.2-.1 2.4-.3 3.5" />
        <path d="M5.6 8.3A7 7 0 0 1 19 10.5c0 1-.05 2-.15 3" />
        <path d="M5 12.5c0 1.6-.3 3.1-.9 4.4" />
        <path d="M14.6 17.5c-.3 1-.7 2-1.1 2.8" />
        <path d="M18.4 16.4c-.2.8-.5 1.6-.8 2.3" />
      </svg>
    </span>
  {/snippet}
  <p class="passkey-lead">
    {t(
      "Save a passkey on this device and you won't need your 24 words to sign in. Your words still work.",
    )}
  </p>
  {#if error}<p class="warning-card">{error}</p>{/if}
  {#snippet footer()}
    <div class="sheet-actions">
      <button class="sheet-submit" type="button" onclick={save} disabled={busy}
        >{busy ? t('Working…') : t('Save passkey')}</button
      >
    </div>
  {/snippet}
</OverlaySheet>

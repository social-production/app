<script lang="ts">
  import { t } from '#lib/i18n/index.ts';
  import OverlaySheet from '#lib/components/overlay/OverlaySheet.svelte';
  import {
    directionsApps,
    platformOf,
    type DirectionsApp,
    type DirectionsTarget,
  } from '#lib/domain/directions.ts';

  /** Directions: pick a map app (owner, 2026-11-18). One row per app, the phone's own first,
   * and Copy address; each opens the app on its own and the sheet closes. */
  let { open, to, onClose }: { open: boolean; to: DirectionsTarget; onClose: () => void } =
    $props();

  const apps = $derived(directionsApps(to, platformOf(navigator.userAgent)));
  let copied = $state(false);

  // Each app's mark: its own colour and initial (no brand artwork).
  const TONES: Record<DirectionsApp['id'], string> = {
    google: '#1a73e8',
    apple: '#3a3a3c',
    waze: '#33ccff',
    phone: 'var(--brand)',
    osm: '#7ebc6f',
  };

  async function copy(): Promise<void> {
    await navigator.clipboard?.writeText(to.label || `${to.latitude}, ${to.longitude}`);
    copied = true;
    setTimeout(() => (copied = false), 1500);
  }
</script>

<OverlaySheet
  {open}
  title={t('Directions')}
  labelledById="directions-title"
  rootClass="shell directions-sheet"
  {onClose}
>
  {#snippet subtitle()}<p class="directions-place">{t(to.label)}</p>{/snippet}
  <ul class="directions-list">
    {#each apps as app (app.id)}
      <li>
        <!-- eslint-disable-next-line svelte/no-navigation-without-resolve -- an outside app -->
        <a href={app.href} target="_blank" rel="noopener noreferrer" onclick={onClose}>
          <span class="directions-mark" style={`--app: ${TONES[app.id]}`} aria-hidden="true">
            {#if app.id === 'phone'}
              <svg viewBox="0 0 24 24"
                ><path d="M12 21s-6-5.6-6-11a6 6 0 1 1 12 0c0 5.4-6 11-6 11z" /><circle
                  cx="12"
                  cy="10"
                  r="2.2"
                /></svg
              >
            {:else}{app.name[0]}{/if}
          </span>
          <span class="directions-name">{app.name}</span>
          <svg class="directions-go" aria-hidden="true" viewBox="0 0 24 24"
            ><path d="M9 6l6 6-6 6" /></svg
          >
        </a>
      </li>
    {/each}
    <li>
      <button type="button" onclick={copy}>
        <span class="directions-mark copy" aria-hidden="true">
          <svg viewBox="0 0 24 24"
            ><rect x="8" y="8" width="12" height="12" rx="2.5" /><path
              d="M16 8V6a2 2 0 0 0-2-2H6a2 2 0 0 0-2 2v8a2 2 0 0 0 2 2h2"
            /></svg
          >
        </span>
        <span class="directions-name">{copied ? t('Copied') : t('Copy address')}</span>
      </button>
    </li>
  </ul>
</OverlaySheet>

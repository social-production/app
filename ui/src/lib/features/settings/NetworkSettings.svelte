<script lang="ts">
  import { t } from '#lib/i18n/index.ts';
  import SettingPanel from '#lib/components/settings/SettingPanel.svelte';
  import { explain } from '#lib/services/errors.ts';
  import {
    networkServers,
    resetNetworkServer,
    saveNetworkServer,
    type NetworkServers,
  } from '#lib/services/network.ts';

  import HelpNetworkSettings from './HelpNetworkSettings.svelte';

  /** Desktop only (ADR 0091): which bootstrap/relay server this node uses, and a field to
   * add one (a community copy, docs/RUN-A-BOOTSTRAP.md). Saving restarts the app. */
  let servers = $state<NetworkServers | null>(null);
  let url = $state('');
  let busy = $state(false);
  let error = $state<string>();

  /** A server's address without its scheme, short enough for a row. */
  const host = (address: string): string => address.replace(/^https?:\/\//, '');
  const canSave = $derived(url.trim().startsWith('https://') && !busy);

  $effect(() => {
    void networkServers().then((s) => (servers = s));
  });

  async function run(action: () => Promise<void>): Promise<void> {
    busy = true;
    error = undefined;
    try {
      await action();
    } catch (e) {
      error = explain(e);
      busy = false;
    }
  }
</script>

{#if servers}
  <section class="settings-section" id="settings-network">
    <h2>{t('Network')}</h2>
    <p class="section-intro">
      {t(
        'This app finds other people through a bootstrap server. It tries yours first, then the built-in list, and uses the first that answers.',
      )}
    </p>
    <div class="card stack flush">
      <SettingPanel
        label={t('In use')}
        value={host(servers.current.bootstrap)}
        hint={servers.fixed ? t('Set when the app was started; it can’t be changed here.') : ''}
      >
        <p class="setting-line server-url">{servers.current.bootstrap}</p>
      </SettingPanel>

      {#if !servers.fixed}
        <SettingPanel
          label={t('Your server')}
          value={servers.saved ? host(servers.saved.bootstrap) : t('None')}
          hint={t('A community copy of the bootstrap server. Saving restarts the app.')}
        >
          <input
            bind:value={url}
            aria-label={t('Server address')}
            placeholder="https://bootstrap.example.org"
            spellcheck="false"
            inputmode="url"
          />
          {#if error}<p class="setting-hint error">{error}</p>{/if}
          {#snippet footer()}
            <div class="sheet-actions">
              {#if servers?.saved}
                <button
                  class="sheet-cancel"
                  disabled={busy}
                  type="button"
                  onclick={() => run(resetNetworkServer)}>{t('Use the built-in list')}</button
                >
              {/if}
              <button
                class="sheet-submit"
                disabled={!canSave}
                type="button"
                onclick={() => run(() => saveNetworkServer(url))}
              >
                {busy ? t('Restarting…') : t('Save and restart')}
              </button>
            </div>
          {/snippet}
        </SettingPanel>
      {/if}
      <HelpNetworkSettings />
    </div>
  </section>
{/if}

<style>
  .server-url {
    overflow-wrap: anywhere;
  }
</style>

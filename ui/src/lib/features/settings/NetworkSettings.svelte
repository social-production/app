<script lang="ts">
  import { explain } from '#lib/services/errors.ts';
  import {
    networkServers,
    resetNetworkServer,
    saveNetworkServer,
    type NetworkServers,
  } from '#lib/services/network.ts';

  /** Desktop only (ADR 0091): which bootstrap/relay server this node uses, and a field to
   * add one (a community copy, docs/RUN-A-BOOTSTRAP.md). Saving restarts the app. */
  let servers = $state<NetworkServers | null>(null);
  let url = $state('');
  let busy = $state(false);
  let error = $state<string>();

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
    <h2>Network</h2>
    <p class="section-intro">
      This app finds other people through a bootstrap server. It tries yours first, then the
      built-in list, and uses the first that answers.
    </p>
    <div class="card stack flush">
      <div class="setting-item">
        <div>
          <strong>In use</strong>
          <p class="server-url">{servers.current.bootstrap}</p>
          {#if servers.fixed}<p>Set when the app was started; it can't be changed here.</p>{/if}
        </div>
      </div>

      {#if !servers.fixed}
        <form
          class="setting-item account-form"
          onsubmit={(e) => {
            e.preventDefault();
            void run(() => saveNetworkServer(url));
          }}
        >
          <div>
            <strong>Your server</strong>
            <p>
              {servers.saved ? servers.saved.bootstrap : 'None saved.'} Saving restarts the app.
            </p>
          </div>
          <label>
            Server address
            <input
              bind:value={url}
              placeholder="https://bootstrap.example.org"
              spellcheck="false"
              inputmode="url"
            />
          </label>
          {#if error}<p class="profile-image-error">{error}</p>{/if}
          <div class="server-actions">
            <button class="button-secondary" disabled={!canSave} type="submit">
              {busy ? 'Restarting…' : 'Save and restart'}
            </button>
            {#if servers.saved}
              <button
                class="button-secondary"
                disabled={busy}
                type="button"
                onclick={() => run(resetNetworkServer)}>Use the built-in list</button
              >
            {/if}
          </div>
        </form>
      {/if}
    </div>
  </section>
{/if}

<style>
  .server-url {
    overflow-wrap: anywhere;
  }
  .server-actions {
    display: flex;
    flex-wrap: wrap;
    gap: 0.5rem;
  }
</style>

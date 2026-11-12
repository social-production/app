<script lang="ts">
  import { explain } from '#lib/services/errors.ts';
  import { helpNetworkState, setHelpNetwork, type HelpState } from '#lib/services/network.ts';

  /** Desktop only (ADR 0094): run a bootstrap server others can reach, and announce it in the
   * directory, so people can still find each other if the built-in servers go. Saving
   * restarts the app. */
  let help = $state<HelpState | null>(null);
  let enabled = $state(false);
  let port = $state('8787');
  let publicUrl = $state('');
  let busy = $state(false);
  let error = $state<string>();

  const portNumber = $derived(Number(port));
  const portOk = $derived(Number.isInteger(portNumber) && portNumber > 0 && portNumber < 65536);
  const addressOk = $derived(/^https?:\/\/\S+$/.test(publicUrl.trim()));
  const canSave = $derived(!busy && (!enabled || (portOk && addressOk)));

  $effect(() => {
    void helpNetworkState().then((state) => {
      help = state;
      if (!state) return;
      enabled = state.settings.enabled;
      port = String(state.settings.port || 8787);
      publicUrl = state.settings.publicUrl;
    });
  });

  async function save(): Promise<void> {
    busy = true;
    error = undefined;
    try {
      await setHelpNetwork({ enabled, port: portNumber, publicUrl: publicUrl.trim() });
    } catch (e) {
      error = explain(e);
      busy = false;
    }
  }
</script>

{#if help}
  <form
    class="setting-item account-form"
    onsubmit={(e) => {
      e.preventDefault();
      void save();
    }}
  >
    <div>
      <strong>Help the network</strong>
      <p>
        Run a bootstrap server on this computer, so new people can find the network even if the
        built-in servers stop. Others must be able to reach it: forward the port on your router (or
        use a server with its own address), and keep the app open.
      </p>
      {#if help.running && help.announce}
        <p class="server-url">Running and announced at {help.announce.bootstrap}</p>
      {:else if help.error}
        <p class="profile-image-error">Not running: {help.error}</p>
      {/if}
    </div>
    <label class="help-toggle">
      <input type="checkbox" bind:checked={enabled} />
      Help the network
    </label>
    {#if enabled}
      <label>
        Port on this computer
        <input bind:value={port} inputmode="numeric" spellcheck="false" />
      </label>
      <label>
        Address other people use
        <input
          bind:value={publicUrl}
          placeholder="http://203.0.113.5:8787"
          spellcheck="false"
          inputmode="url"
        />
      </label>
    {/if}
    {#if error}<p class="profile-image-error">{error}</p>{/if}
    <div>
      <button class="button-secondary" disabled={!canSave} type="submit">
        {busy ? 'Restarting…' : 'Save and restart'}
      </button>
    </div>
  </form>
{/if}

<style>
  .server-url {
    overflow-wrap: anywhere;
  }
  .help-toggle {
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }
</style>

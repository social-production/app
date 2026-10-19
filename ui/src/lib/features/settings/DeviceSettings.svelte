<script lang="ts">
  import CopyKey from '#lib/components/CopyKey.svelte';
  import { displayName, type Person } from '#lib/domain/people.ts';
  import { explain } from '#lib/services/errors.ts';
  import {
    deviceRequests,
    devicesOf,
    getPerson,
    writeDeviceLink,
    type DeviceLink,
  } from '#lib/services/people.ts';

  let { me }: { me: string } = $props();

  let paired = $state<Person[]>([]);
  let requests = $state<{ link: DeviceLink; from: Person }[]>([]);
  let mainKey = $state('');
  let status = $state<string>();
  let error = $state<string>();

  const canSend = $derived(mainKey.trim().startsWith('uhCAk') && mainKey.trim() !== me);

  async function load(): Promise<void> {
    try {
      const [agents, links] = await Promise.all([devicesOf(me), deviceRequests()]);
      paired = await Promise.all(agents.map(getPerson));
      requests = await Promise.all(
        links.map(async (link) => ({
          link,
          from: await getPerson(link.device === me ? link.primary : link.device),
        })),
      );
    } catch (e) {
      error = explain(e);
    }
  }

  async function act(link: DeviceLink, done: string): Promise<void> {
    error = undefined;
    try {
      await writeDeviceLink(link);
      status = done;
      mainKey = '';
      await load();
    } catch (e) {
      error = explain(e);
    }
  }

  $effect(() => {
    void load();
  });
</script>

<section class="settings-section" id="settings-devices">
  <h2>Devices</h2>
  <p class="section-intro">
    Each app install has its own key. Pair it with your main account so people see one person.
  </p>
  <div class="card stack flush">
    <div class="setting-item">
      <div>
        <strong>Paired devices</strong>
        {#each paired as device (device.agent)}
          <p>{displayName(device)}</p>
        {:else}
          <p>No other devices paired yet.</p>
        {/each}
      </div>
    </div>

    {#each requests as request (request.link.device + request.link.primary)}
      <div class="setting-item">
        <div>
          <strong>Pairing request</strong>
          <p>From {displayName(request.from)}</p>
        </div>
        <button
          class="button-primary"
          type="button"
          onclick={() => act(request.link, 'Device paired.')}>Confirm</button
        >
      </div>
    {/each}

    <div class="setting-item">
      <div>
        <strong>This device's key</strong>
        <p>Paste it on your main account, or paste the main account's key below.</p>
      </div>
      <CopyKey value={me} label="Copy this device's key" />
    </div>

    <form
      class="setting-item account-form"
      onsubmit={(e) => {
        e.preventDefault();
        void act(
          { primary: mainKey.trim(), device: me },
          'Request sent. Confirm it on your main account.',
        );
      }}
    >
      <div>
        <strong>Pair with my main account</strong>
        <p>Paste the key from "Copy my key" on your main account.</p>
      </div>
      <label>
        Main account key
        <input bind:value={mainKey} placeholder="uhCAk…" spellcheck="false" />
      </label>
      <p class:profile-image-error={!!error} class:bio-saved={!error} class="status-line">
        {error ?? status ?? ''}
      </p>
      <button class="button-secondary" disabled={!canSend} type="submit"
        >Send pairing request</button
      >
    </form>
  </div>
</section>

<style>
  .status-line {
    min-height: 1.5em;
    margin: 0;
  }
</style>

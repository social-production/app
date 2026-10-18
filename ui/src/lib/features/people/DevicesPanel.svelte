<script lang="ts">
  import CopyKey from '#lib/components/CopyKey.svelte';
  import { displayName, type Person } from '#lib/domain/people.ts';
  import { explain } from '#lib/services/account.ts';
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

<section class="panel stack">
  <h2>Your devices</h2>
  <p class="muted">
    Each app install has its own key. Pair it with your main account so people see one person.
  </p>
  {#each paired as device (device.agent)}
    <p>Paired: <strong>{displayName(device)}</strong></p>
  {:else}
    <p class="muted">No other devices paired yet.</p>
  {/each}

  {#each requests as request (request.link.device + request.link.primary)}
    <div class="row">
      <span>Pairing request from <strong>{displayName(request.from)}</strong></span>
      <button class="primary" onclick={() => act(request.link, 'Device paired.')}>Confirm</button>
    </div>
  {/each}

  <CopyKey value={me} label="Copy this device's key" />
  <label>
    Pair this device with my main account (paste its key from "Copy my key")
    <input bind:value={mainKey} placeholder="uhCAk…" spellcheck="false" />
  </label>
  <button
    disabled={!mainKey.trim().startsWith('uhCAk') || mainKey.trim() === me}
    onclick={() =>
      act(
        { primary: mainKey.trim(), device: me },
        'Request sent. Confirm it on your main account.',
      )}>Send pairing request</button
  >
  <p class="status" class:error={!!error}>{error ?? status ?? ''}</p>
</section>

<style>
  .status {
    min-height: 1.5em;
    margin: 0;
    color: var(--text-soft);
  }
</style>

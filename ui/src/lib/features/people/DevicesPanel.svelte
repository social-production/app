<script lang="ts">
  import { keyTag } from '#lib/domain/people.ts';
  import { explain } from '#lib/services/account.ts';
  import {
    deviceRequests,
    devicesOf,
    writeDeviceLink,
    type DeviceLink,
  } from '#lib/services/people.ts';

  let { me }: { me: string } = $props();

  let paired = $state<string[]>([]);
  let requests = $state<DeviceLink[]>([]);
  let mainKey = $state('');
  let status = $state<string>();
  let error = $state<string>();

  async function load(): Promise<void> {
    try {
      [paired, requests] = await Promise.all([devicesOf(me), deviceRequests()]);
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
  {#each paired as device (device)}
    <p>Paired: <code>{keyTag(device)}</code> <span class="muted">{device}</span></p>
  {:else}
    <p class="muted">No other devices paired yet.</p>
  {/each}

  {#each requests as request (request.device + request.primary)}
    <div class="row">
      <span
        >Pairing request from <code
          >{keyTag(request.device === me ? request.primary : request.device)}</code
        ></span
      >
      <button class="primary" onclick={() => act(request, 'Device paired.')}>Confirm</button>
    </div>
  {/each}

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
  {#if status}<p class="muted">{status}</p>{/if}
  {#if error}<p class="error">{error}</p>{/if}
</section>

<style>
  code {
    font-family: ui-monospace, monospace;
  }
  .muted {
    word-break: break-all;
  }
</style>

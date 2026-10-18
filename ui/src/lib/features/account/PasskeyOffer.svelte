<script lang="ts">
  import { addPasskey, devicePasskeysAvailable, explain } from '#lib/services/account.ts';

  let { name }: { name: string } = $props();

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
      await addPasskey(name);
      close('saved');
    } catch (e) {
      error = explain(e);
    } finally {
      busy = false;
    }
  }
</script>

{#if visible}
  <section class="panel stack">
    <strong>Sign in faster on this device</strong>
    <p class="muted">
      Save a passkey here (fingerprint, face or screen lock) so you don't need your 24 words next
      time. Optional — your words always work.
    </p>
    {#if error}<p class="error">{error}</p>{/if}
    <div class="row">
      <button class="primary" onclick={save} disabled={busy}>Save a passkey</button>
      <button onclick={() => close('dismissed')}>Not now</button>
    </div>
  </section>
{/if}

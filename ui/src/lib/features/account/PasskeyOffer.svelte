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
  <section class="empty-card passkey-offer">
    <strong>Sign in faster on this device</strong>
    <p>
      Save a passkey here (fingerprint, face or screen lock) so you don't need your 24 words next
      time. Optional — your words always work.
    </p>
    {#if error}<p class="warning-card">{error}</p>{/if}
    <div class="button-row">
      <button class="button-primary" type="button" onclick={save} disabled={busy}
        >Save a passkey</button
      >
      <button class="button-secondary" type="button" onclick={() => close('dismissed')}
        >Not now</button
      >
    </div>
  </section>
{/if}

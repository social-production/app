<script lang="ts">
  import CopyKey from '#lib/components/CopyKey.svelte';
  import ConfirmDialog from '#lib/components/overlay/ConfirmDialog.svelte';
  import {
    addPasskey,
    devicePasskeysAvailable,
    inNativeApp,
    listedOnNetwork,
    signOut,
  } from '#lib/services/account.ts';
  import { explain } from '#lib/services/errors.ts';
  import { viewer } from '#lib/stores/viewer.svelte.ts';

  let { me }: { me: string } = $props();

  let passkeys = $state(false);
  let busy = $state(false);
  let message = $state<string>();
  let error = $state<string>();
  /** Desktop only: no other node lists the account yet (ADR 0092). */
  let unheld = $state(false);

  $effect(() => {
    void devicePasskeysAvailable().then((available) => (passkeys = available));
  });

  async function savePasskey(): Promise<void> {
    busy = true;
    error = message = undefined;
    try {
      await addPasskey(viewer.name || 'Social Production');
      message = 'Passkey saved on this device.';
    } catch (e) {
      error = explain(e);
    } finally {
      busy = false;
    }
  }

  async function leave(): Promise<void> {
    if (inNativeApp() && (await listedOnNetwork(me, viewer.name)) !== true) {
      unheld = true;
      return;
    }
    await signOut();
  }
</script>

<section class="settings-section" id="settings-account">
  <h2>Account</h2>
  <div class="card stack flush">
    <div class="setting-item">
      <div>
        <strong>Your key</strong>
        <p>Others paste it to vouch for you or pair a device.</p>
      </div>
      <CopyKey value={me} />
    </div>

    {#if passkeys}
      <div class="setting-item">
        <div>
          <strong>Passkey</strong>
          <p>
            Sign in on this device with fingerprint, face or screen lock. Your 24 words always work
            too.
          </p>
          {#if error}<p class="profile-image-error">{error}</p>{:else if message}<p
              class="bio-saved"
            >
              {message}
            </p>{/if}
        </div>
        <button class="button-secondary" disabled={busy} type="button" onclick={savePasskey}>
          {busy ? 'Waiting…' : 'Save a passkey'}
        </button>
      </div>
    {/if}

    <div class="setting-item">
      <div>
        <strong>Sign out</strong>
        <p>Sign out on this device. Sign back in with your 24 words.</p>
      </div>
      <button class="button-secondary" type="button" onclick={leave}>Sign out</button>
    </div>
  </div>
</section>

<ConfirmDialog
  open={unheld}
  title="Your account may not have reached the network yet"
  body="No other node lists it yet, and this computer may hold the only copy. If you sign out now, your profile and history could be lost: your words would bring back an empty account. Wait a few minutes and try again, or sign out anyway."
  confirmLabel="Sign out anyway"
  danger
  onSettle={(ok) => (ok ? void signOut() : (unheld = false))}
/>

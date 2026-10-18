<script lang="ts">
  import CopyKey from '#lib/components/CopyKey.svelte';
  import { addPasskey, devicePasskeysAvailable, explain, signOut } from '#lib/services/account.ts';
  import { viewer } from '#lib/stores/viewer.svelte.ts';

  let { me }: { me: string } = $props();

  let passkeys = $state(false);
  let busy = $state(false);
  let message = $state<string>();
  let error = $state<string>();

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
      <button class="button-secondary" type="button" onclick={() => signOut()}>Sign out</button>
    </div>
  </div>
</section>

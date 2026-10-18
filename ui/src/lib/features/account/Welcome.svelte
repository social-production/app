<script lang="ts">
  import { explain, passkeysSupported, signIn } from '#lib/services/account.ts';

  import RestoreFlow from './RestoreFlow.svelte';
  import SignupFlow from './SignupFlow.svelte';

  let mode = $state<'choose' | 'signup' | 'restore'>('choose');
  let busy = $state(false);
  let error = $state<string>();

  async function login(): Promise<void> {
    busy = true;
    error = undefined;
    try {
      await signIn(() => {});
    } catch (e) {
      error = explain(e);
    } finally {
      busy = false;
    }
  }
</script>

<section class="panel stack welcome">
  {#if !passkeysSupported()}
    <p class="error">
      This browser can't use passkeys. Please update it, or use Chrome, Safari, Firefox or Edge.
    </p>
  {:else if mode === 'signup'}
    <SignupFlow onCancel={() => (mode = 'choose')} />
  {:else if mode === 'restore'}
    <RestoreFlow onCancel={() => (mode = 'choose')} />
  {:else}
    <h1>Social Production</h1>
    <p class="muted">Organise projects and share the means of production — no central owner.</p>
    <button class="primary" onclick={() => (mode = 'signup')}>Create an account</button>
    <button onclick={login} disabled={busy}>{busy ? 'Waiting for passkey…' : 'Sign in'}</button>
    <button onclick={() => (mode = 'restore')}>Restore with recovery phrase</button>
    {#if error}<p class="error">{error}</p>{/if}
  {/if}
</section>

<style>
  .welcome {
    max-width: 560px;
    margin: 24px auto;
  }
</style>

<script lang="ts">
  import { devicePasskeysAvailable, explain, signInWithPasskey } from '#lib/services/account.ts';

  import PhraseSignIn from './PhraseSignIn.svelte';
  import SignupFlow from './SignupFlow.svelte';

  let mode = $state<'choose' | 'signup' | 'phrase'>('choose');
  let busy = $state(false);
  let error = $state<string>();
  let passkeys = $state(false);

  $effect(() => {
    void devicePasskeysAvailable().then((available) => (passkeys = available));
  });

  async function passkeyLogin(): Promise<void> {
    busy = true;
    error = undefined;
    try {
      await signInWithPasskey(() => {});
    } catch (e) {
      error = `${explain(e)} You can always sign in with your recovery phrase.`;
    } finally {
      busy = false;
    }
  }
</script>

<section class="panel stack welcome">
  {#if mode === 'signup'}
    <SignupFlow onCancel={() => (mode = 'choose')} />
  {:else if mode === 'phrase'}
    <PhraseSignIn onCancel={() => (mode = 'choose')} />
  {:else}
    <h1>Social Production</h1>
    <p class="muted">Organise projects and share the means of production — no central owner.</p>
    <button class="primary" onclick={() => (mode = 'signup')}>Create an account</button>
    <button onclick={() => (mode = 'phrase')}>Sign in with recovery phrase</button>
    {#if passkeys}
      <button onclick={passkeyLogin} disabled={busy}>
        {busy ? 'Waiting for your passkey…' : 'Sign in with a passkey'}
      </button>
    {/if}
    {#if error}<p class="error">{error}</p>{/if}
  {/if}
</section>

<style>
  .welcome {
    max-width: 560px;
    margin: 24px auto;
  }
</style>

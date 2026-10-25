<script lang="ts">
  import { devicePasskeysAvailable, signInWithPasskey } from '#lib/services/account.ts';
  import { explain } from '#lib/services/errors.ts';

  import PhraseSignIn from './PhraseSignIn.svelte';
  import SignupFlow from './SignupFlow.svelte';

  import '#lib/components/styles/onboarding.css';

  /** web's onboarding page: Sign up / Log in chips over one panel. */
  let mode = $state<'signup' | 'login'>('signup');
  let busy = $state(false);
  let error = $state<string>();
  let passkeys = $state(false);

  const modes = [
    {
      value: 'signup',
      label: 'Sign up',
      description: 'New here? You get 24 words that are your account.',
    },
    {
      value: 'login',
      label: 'Log in',
      description: 'Type your 24 words, or use a passkey saved on this device.',
    },
  ] as const;
  const active = $derived(modes.find((m) => m.value === mode));

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

<div class="onboarding">
  <section class="page">
    <section class="hero-card">
      <h1>Sign in or create an account</h1>
      <p>Sign in to post, follow people, and create projects, threads, and events.</p>
    </section>

    <section class="panel">
      <p class="mode-hint">
        Choose <strong>Sign up</strong> for a new account, or <strong>Log in</strong> if you already have
        one.
      </p>
      <div class="choice-row" role="tablist" aria-label="Account mode">
        {#each modes as option (option.value)}
          <button
            class:active={mode === option.value}
            class="toggle-chip"
            type="button"
            role="tab"
            aria-selected={mode === option.value}
            onclick={() => (mode = option.value)}>{option.label}</button
          >
        {/each}
      </div>
      <p class="helper-text">{active?.description}</p>

      {#if mode === 'signup'}
        <SignupFlow />
      {:else}
        {#if passkeys}
          <div class="button-row">
            <button class="button-secondary" type="button" onclick={passkeyLogin} disabled={busy}>
              {busy ? 'Waiting for your passkey…' : 'Log in with a passkey'}
            </button>
          </div>
        {/if}
        {#if error}<p class="status-note">{error}</p>{/if}
        <PhraseSignIn />
      {/if}
    </section>
  </section>
</div>

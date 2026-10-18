<script lang="ts">
  import {
    explain,
    firstInvalidWord,
    phraseIsValid,
    signInWithPhrase,
    splitWords,
    type AccountStep,
  } from '#lib/services/account.ts';

  import ProgressSteps from './ProgressSteps.svelte';

  let { onCancel }: { onCancel: () => void } = $props();

  let text = $state('');
  let working = $state(false);
  let progress = $state<AccountStep>('connecting');
  let error = $state<string>();

  const words = $derived(splitWords(text));
  const valid = $derived(phraseIsValid(words));
  const hint = $derived.by(() => {
    const unknown = firstInvalidWord(words);
    if (unknown) return `${unknown} isn't in the word list — check its spelling.`;
    if (words.length < 24)
      return `${24 - words.length} more word${words.length === 23 ? '' : 's'} needed.`;
    if (words.length > 24) return `That's ${words.length} words; a phrase has 24.`;
    if (!valid) return 'All words are real, but they are not a valid phrase — check their order.';
    return 'Looks right.';
  });

  async function go(): Promise<void> {
    working = true;
    error = undefined;
    try {
      await signInWithPhrase(words, (s) => (progress = s));
    } catch (e) {
      error = explain(e);
      working = false;
    }
  }
</script>

{#if working}
  <ProgressSteps current={progress} />
{:else}
  <h2>Sign in with your recovery phrase</h2>
  <p class="muted">
    Type or paste your 24 words. This device stays signed in until you sign out. If your account
    lives on another site or app, it is brought here — same identity.
  </p>
  <label>
    Recovery phrase
    <textarea bind:value={text} rows="5" autocapitalize="none" spellcheck="false"></textarea>
  </label>
  <p class="muted">{words.length}/24 words · {hint}</p>
  {#if error}<p class="error">{error}</p>{/if}
  <div class="row">
    <button class="primary" disabled={!valid} onclick={go}>Sign in</button>
    <button onclick={onCancel}>Back</button>
  </div>
{/if}

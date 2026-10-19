<script lang="ts">
  import {
    firstInvalidWord,
    phraseIsValid,
    signInWithPhrase,
    splitWords,
    type AccountStep,
  } from '#lib/services/account.ts';
  import { explain } from '#lib/services/errors.ts';

  import ProgressSteps from './ProgressSteps.svelte';

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

  async function go(event: SubmitEvent): Promise<void> {
    event.preventDefault();
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
  <form class="stack" onsubmit={go}>
    <label>
      <span class="field-label">Recovery phrase</span>
      <textarea bind:value={text} rows="5" autocapitalize="none" spellcheck="false"></textarea>
    </label>
    <p class="helper-text">{words.length}/24 words · {hint}</p>
    <div class="button-row">
      <button class="button-primary" disabled={!valid} type="submit">Log in</button>
    </div>
    {#if error}<p class="status-note">{error}</p>{/if}
    <p class="helper-text">
      This device stays signed in until you sign out. An account from another site or app comes
      along — same identity.
    </p>
  </form>
{/if}

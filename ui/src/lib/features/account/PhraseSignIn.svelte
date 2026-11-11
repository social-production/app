<script lang="ts">
  import {
    ACCOUNT_MOVED,
    firstInvalidWord,
    GatewayError,
    inNativeApp,
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
  /** The account moved to the desktop app (ADR 0092): offer to host it here again. */
  let moved = $state(false);

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
    await open(false);
  }

  async function open(reclaim: boolean): Promise<void> {
    working = true;
    error = undefined;
    moved = false;
    try {
      await signInWithPhrase(words, (s) => (progress = s), { reclaim });
    } catch (e) {
      error = explain(e);
      moved = e instanceof GatewayError && e.status === ACCOUNT_MOVED;
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
    {#if moved}
      <div class="button-row">
        <button class="button-secondary" type="button" onclick={() => open(true)}>
          Use it here instead
        </button>
      </div>
    {/if}
    <p class="helper-text">
      This device stays signed in until you sign out. An account from another site or app comes
      along — same identity.
    </p>
    {#if inNativeApp()}
      <!-- ADR 0090: one agent on two nodes at once can fork its chain. -->
      <p class="helper-text">
        Your account now lives on this computer. If you used it on a website, the website stops
        hosting it once your history arrives here (you can take it back there with your words).
        Logging in can take up to two minutes while your history arrives from the network.
      </p>
    {/if}
  </form>
{/if}

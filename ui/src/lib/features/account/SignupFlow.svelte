<script lang="ts">
  import { t } from '#lib/i18n/index.ts';
  import PhraseGrid from '#lib/components/PhraseGrid.svelte';
  import { HANDLE_MAX, handleProblem } from '#lib/domain/handle.ts';
  import {
    confirmationPositions,
    createAccount,
    newPhrase,
    type AccountStep,
  } from '#lib/services/account.ts';
  import { explain } from '#lib/services/errors.ts';

  import ProgressSteps from './ProgressSteps.svelte';

  const words = newPhrase();
  const positions = confirmationPositions();

  let step = $state<'name' | 'phrase' | 'confirm' | 'working'>('name');
  let name = $state('');
  const problem = $derived(handleProblem(name));
  let answers = $state(['', '', '']);
  let progress = $state<AccountStep>('checking');
  let error = $state<string>();
  let copied = $state(false);

  /** The 24 words on the clipboard, in order, for pasting into a document or password manager. */
  async function copyWords(): Promise<void> {
    try {
      await navigator.clipboard.writeText(words.join(' '));
      copied = true;
    } catch {
      copied = false;
    }
  }

  const confirmed = $derived(
    positions.every((p, i) => answers[i]?.trim().toLowerCase() === words[p]),
  );

  async function finish(): Promise<void> {
    step = 'working';
    error = undefined;
    try {
      await createAccount(name.trim(), words, (s) => (progress = s));
    } catch (e) {
      error = explain(e);
      step = 'confirm';
    }
  }
</script>

{#if step !== 'working'}
  <div class="ob-steps" aria-hidden="true">
    <i class:on={true}></i><i class:on={step !== 'name'}></i><i class:on={step === 'confirm'}></i>
  </div>
{/if}
{#if step === 'name'}
  <form class="stack" data-step="name" onsubmit={(e) => (e.preventDefault(), (step = 'phrase'))}>
    <label>
      <span class="field-label">{t('Username')}</span>
      <input
        bind:value={name}
        maxlength={HANDLE_MAX}
        autocomplete="username"
        autocapitalize="none"
      />
    </label>
    <!-- One reserved line (as on web), so typing doesn't push the button down. -->
    <div class="handle-line">
      {#if problem && name.trim().length >= 3}<p class="status-note">{problem}</p>{/if}
    </div>
    <p class="ob-note">
      {t('Next you get 24 words. They are your account; keep them somewhere safe.')}
    </p>
    <div class="button-row">
      <button class="button-primary" disabled={!!problem} type="submit">{t('Next')}</button>
    </div>
  </form>
{:else if step === 'phrase'}
  <div class="stack" data-step="phrase">
    <p class="field-label">{t('Your recovery phrase')}</p>
    <p class="ob-phrase-note">
      {t('These 24 words')} <strong>{t('are')}</strong>
      {t(
        'your account. Write them down and keep them safe: anyone with them can act as you, and without them a lost device means a lost account.',
      )}
    </p>
    <PhraseGrid {words} />
    <div class="button-row">
      <button class="button-secondary copy-words" type="button" onclick={() => void copyWords()}>
        {copied ? t('Copied all 24 words') : t('Copy all 24 words')}
      </button>
    </div>
    <div class="button-row">
      <button class="button-primary" type="button" onclick={() => (step = 'confirm')}
        >{t("I've written them down")}</button
      >
    </div>
  </div>
{:else if step === 'confirm'}
  <form class="stack" data-step="confirm" onsubmit={(e) => (e.preventDefault(), void finish())}>
    <p class="field-label">{t('Check your phrase')}</p>
    {#each positions as position, i (position)}
      <label>
        <span class="field-label">Word #{position + 1}</span>
        <input
          bind:value={answers[i]}
          autocapitalize="none"
          autocomplete="off"
          spellcheck="false"
        />
      </label>
    {/each}
    {#if error}<p class="status-note">{error}</p>{/if}
    <div class="button-row">
      <button class="button-secondary" type="button" onclick={() => (step = 'phrase')}
        >{t('Show the words again')}</button
      >
    </div>
    <div class="button-row">
      <button class="button-primary" disabled={!confirmed} type="submit"
        >{t('Create account')}</button
      >
    </div>
  </form>
{:else}
  <ProgressSteps current={progress} />
{/if}

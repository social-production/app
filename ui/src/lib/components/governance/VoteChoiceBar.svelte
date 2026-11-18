<script lang="ts">
  import { t } from '#lib/i18n/index.ts';
  import type { Snippet } from 'svelte';
  import type { VoteChoice } from '#lib/domain/governance.ts';

  /**
   * web's VoteChoiceBar: No, then Yes. Votes are final here (ADR 0033), so once cast the bar
   * says how you voted. `docked` puts it in a sheet's joined dock (owner, 2026-11-18), with
   * `lead` (the vote sheet's Next) before it.
   */
  interface Props {
    mine: VoteChoice | null;
    onChoose: (choice: VoteChoice) => void;
    docked?: boolean;
    /** Not yet: a plan's vote waits for every question's answer. */
    disabled?: boolean;
    lead?: Snippet;
  }

  let { mine, onChoose, docked = false, disabled = false, lead }: Props = $props();
</script>

<div class="choice-bar" class:sheet-actions={docked} class:chosen={!!mine}>
  {@render lead?.()}
  {#if mine}
    <button class="choice voted" type="button" disabled aria-pressed="true"
      >✓ {mine === 'Yes' ? t('You voted Yes') : t('You voted No')}</button
    >
  {:else}
    <button
      class="choice no"
      type="button"
      aria-pressed="false"
      {disabled}
      onclick={() => onChoose('No')}>{t('No')}</button
    >
    <button
      class="choice yes"
      class:sheet-submit={docked}
      type="button"
      aria-pressed="false"
      {disabled}
      onclick={() => onChoose('Yes')}>{t('Yes')}</button
    >
  {/if}
</div>

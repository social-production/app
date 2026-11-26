<script lang="ts">
  import { t } from '#lib/i18n/index.ts';
  import RoundPlusButton from '#lib/components/RoundPlusButton.svelte';

  import PhaseMenu, { type PhaseMenuItem } from './PhaseMenu.svelte';

  /**
   * The dock's one green add action (owner-approved mockup, 2026-11-26: "a bunch of pluses" was
   * wrong). One thing to add is a plain "+ Add activity"; several are "+ Add ▾", a sheet of them,
   * so a software service's dock keeps its words instead of "+ +".
   */
  interface Props {
    items: PhaseMenuItem[];
    /** The lone activity's button label ("Add activity"); any other lone item shows its own. */
    single: string;
    active?: boolean;
  }

  let { items, single, active = false }: Props = $props();

  const lone = $derived(items[0]?.id === 'activity' ? single : (items[0]?.label ?? single));
</script>

{#if items.length === 1}
  <RoundPlusButton
    standout
    {active}
    label={lone}
    ariaLabel={lone}
    onclick={() => items[0]?.onSelect?.()}
  />
{:else if items.length > 1}
  <PhaseMenu
    {items}
    standout
    label={t('Add')}
    sheetTitle={t('Add')}
    labelledById="add-menu-title"
  />
{/if}

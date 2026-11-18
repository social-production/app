<script lang="ts">
  import { t } from '#lib/i18n/index.ts';
  import OverlaySheet from '#lib/components/overlay/OverlaySheet.svelte';
  import RadiusChips from './RadiusChips.svelte';
  import '#lib/components/styles/map.css';

  /** web's `components/shared/RadiusCombobox.svelte` as a button that opens a Distance sheet:
   * the same one-tap chips and "Other distance" box as the Region filter (owner, 2026-11-17:
   * the drop-down list didn't look like the rest of the app's filters). The map stays in view
   * behind it and follows each tap; Done closes it. */
  interface Props {
    value: string;
    onChange: (value: string) => void;
    ariaLabel?: string;
    options?: { value: string; label: string }[];
    compact?: boolean;
    /** Kept for callers from the drop-down days; the sheet needs neither. */
    portaled?: boolean;
    preferAbove?: boolean;
  }

  let {
    value,
    onChange,
    ariaLabel = 'Search radius',
    options = [],
    compact = false,
  }: Props = $props();

  let open = $state(false);

  const display = $derived.by(() => {
    if (value === 'global') return 'Global';
    const parsed = Number(value);
    return Number.isFinite(parsed) ? `${Math.round(parsed)} km` : value;
  });
</script>

<div class="radius-combobox" class:compact>
  <div class="input-shell" class:open>
    <button
      class="radius-input radius-trigger"
      type="button"
      aria-label={ariaLabel}
      aria-haspopup="dialog"
      aria-expanded={open}
      onclick={() => (open = true)}
    >
      <span>{display}</span>
      <svg class="chevron" aria-hidden="true" viewBox="0 0 24 24" width="14" height="14">
        <path d="M6 9l6 6 6-6" fill="none" stroke="currentColor" stroke-width="2.4" />
      </svg>
    </button>
  </div>
</div>

<OverlaySheet
  {open}
  title={t('Distance')}
  labelledById="radius-sheet-title"
  rootClass="shell feed-filter-sheet radius-sheet"
  onClose={() => (open = false)}
>
  <div class="feed-filter-groups">
    <section>
      <RadiusChips {value} {options} ariaLabel={t('Distance')} {onChange} />
    </section>
  </div>
  {#snippet footer()}
    <div class="sheet-actions">
      <button class="sheet-submit" type="button" onclick={() => (open = false)}>{t('Done')}</button>
    </div>
  {/snippet}
</OverlaySheet>

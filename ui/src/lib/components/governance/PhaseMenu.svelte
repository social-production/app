<script lang="ts" module>
  export interface PhaseMenuItem {
    id: string;
    glyph: string;
    label: string;
    detail: string;
    disabled?: boolean;
    onSelect?: () => void;
  }
</script>

<script lang="ts">
  import { t } from '#lib/i18n/index.ts';
  import OverlaySheet from '#lib/components/overlay/OverlaySheet.svelte';

  import '#lib/components/styles/governance.css';

  /** web's PhaseMenu: every phase move behind one button, so the bar keeps one clear work action. */
  interface Props {
    items: PhaseMenuItem[];
    currentPhaseLabel?: string;
    labelledById?: string;
    label?: string;
    sheetTitle?: string;
    /** Opens the caller's own sheet instead of this list (the one-sheet phase move). */
    onOpen?: () => void;
  }

  let {
    items,
    currentPhaseLabel = '',
    labelledById = 'phase-menu-title',
    label = 'Phase',
    sheetTitle = 'Change phase',
    onOpen,
  }: Props = $props();

  let open = $state(false);
  const hasChoice = $derived(items.some((item) => !item.disabled));

  function choose(item: PhaseMenuItem): void {
    if (item.disabled) return;
    open = false;
    item.onSelect?.();
  }
</script>

{#if items.length > 0}
  <!-- A styling scope only (display: contents), so the trigger itself packs into toolbars. -->
  <span class="phase-menu phase-menu-scope">
    <button
      class="phase-menu-trigger"
      class:muted={!hasChoice}
      data-participation-action="propose-advance"
      type="button"
      aria-haspopup="dialog"
      aria-expanded={open}
      title={sheetTitle}
      onclick={() => (onOpen ? onOpen() : (open = true))}
    >
      <span class="label">{t(label)}</span>
      <svg class="caret" aria-hidden="true" viewBox="0 0 24 24"
        ><path
          d="m6 9 6 6 6-6"
          fill="none"
          stroke="currentColor"
          stroke-width="2"
          stroke-linecap="round"
          stroke-linejoin="round"
        /></svg
      >
    </button>
  </span>

  <OverlaySheet
    {open}
    title={sheetTitle}
    {labelledById}
    rootClass="phase-menu"
    onClose={() => (open = false)}
  >
    <div class="phase-menu-body">
      {#if currentPhaseLabel}
        <p class="phase-menu-current">{t('Now in')} <strong>{t(currentPhaseLabel)}</strong></p>
      {/if}
      <ul class="phase-menu-list">
        {#each items as item (item.id)}
          <li>
            <button
              class="phase-menu-item"
              type="button"
              disabled={item.disabled}
              onclick={() => choose(item)}
            >
              <span class="glyph" aria-hidden="true">{item.disabled ? '🔒' : item.glyph}</span>
              <span class="text">
                <strong>{t(item.label)}</strong>
                <span>{item.detail}</span>
              </span>
            </button>
          </li>
        {/each}
      </ul>
    </div>
  </OverlaySheet>
{/if}

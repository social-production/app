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
  import OverlaySheet from '#lib/components/overlay/OverlaySheet.svelte';

  import '#lib/components/styles/governance.css';

  /** web's PhaseMenu: every phase move behind one button, so the bar keeps one clear work action. */
  interface Props {
    items: PhaseMenuItem[];
    currentPhaseLabel?: string;
    labelledById?: string;
    label?: string;
    sheetTitle?: string;
  }

  let {
    items,
    currentPhaseLabel = '',
    labelledById = 'phase-menu-title',
    label = 'Phase',
    sheetTitle = 'Change phase',
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
  <span class="phase-menu">
    <button
      class="phase-menu-trigger"
      class:muted={!hasChoice}
      data-participation-action="propose-advance"
      type="button"
      aria-haspopup="dialog"
      aria-expanded={open}
      title={sheetTitle}
      onclick={() => (open = true)}
    >
      <span class="label">{label}</span>
      <span class="caret" aria-hidden="true">▾</span>
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
        <p class="phase-menu-current">Now in <strong>{currentPhaseLabel}</strong></p>
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
                <strong>{item.label}</strong>
                <span>{item.detail}</span>
              </span>
            </button>
          </li>
        {/each}
      </ul>
    </div>
  </OverlaySheet>
{/if}

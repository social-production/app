<script lang="ts">
  import { t } from '#lib/i18n/index.ts';
  import ToolbarIcon from '#lib/components/icons/ToolbarIcon.svelte';
  import '#lib/components/styles/governance.css';

  /** web's RoundPlusButton: a round "+" with an optional label ("Vote", "Add update"). */
  interface Props {
    label?: string;
    ariaLabel?: string;
    active?: boolean;
    standout?: boolean;
    /** Show the "+" (only for buttons that add something; owner, 2026-11-23). */
    plus?: boolean;
    /** An icon in place of the "+", kept when a tight dock drops the label. */
    icon?: string;
    onclick: () => void;
  }

  let {
    label = '',
    ariaLabel = 'Add item',
    active = false,
    standout = false,
    plus = true,
    icon,
    onclick,
  }: Props = $props();
</script>

<button
  aria-label={label || ariaLabel}
  aria-pressed={active}
  class:active
  class:standout
  class:with-label={!!label}
  class:no-plus={!plus && !icon}
  class="round-plus-button"
  type="button"
  {onclick}
>
  {#if icon}<ToolbarIcon name={icon} />{:else if plus}<span aria-hidden="true" class="plus-glyph"
      >+</span
    ><span class="plus-icon"><ToolbarIcon name="plus" /></span>{/if}
  {#if label}<span class="plus-label">{t(label)}</span>{/if}
</button>

<script lang="ts">
  import '#lib/components/styles/governance.css';

  /** web's RoundPlusButton: a round "+" with an optional label ("Vote", "Add update"). */
  interface Props {
    label?: string;
    ariaLabel?: string;
    active?: boolean;
    standout?: boolean;
    onclick: () => void;
  }

  let {
    label = '',
    ariaLabel = 'Add item',
    active = false,
    standout = false,
    onclick,
  }: Props = $props();
</script>

<button
  aria-label={label || ariaLabel}
  aria-pressed={active}
  class:active
  class:standout
  class:with-label={!!label}
  class="round-plus-button"
  type="button"
  {onclick}
>
  <span aria-hidden="true" class="plus-glyph">+</span>
  {#if label}<span class="plus-label">{label}</span>{/if}
</button>

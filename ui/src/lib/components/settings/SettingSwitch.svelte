<script lang="ts">
  import { t } from '#lib/i18n/index.ts';
  /** A setting that is on or off, as a row: its name at the left, a switch at the right
   * (Remaining Screens, Settings). `note` is one quiet line under the name. */
  interface Props {
    label: string;
    note?: string;
    checked: boolean;
    disabled?: boolean;
    onToggle: () => void;
  }

  let { label, note = '', checked, disabled = false, onToggle }: Props = $props();
</script>

<div class="set-row set-switch">
  <span class="set-row-label">
    {label}
    {#if note}<small>{note}</small>{/if}
  </span>
  <button
    aria-checked={checked}
    aria-label={label}
    class="switch"
    class:on={checked}
    {disabled}
    role="switch"
    type="button"
    onclick={onToggle}
  >
    <span class="switch-thumb"></span>
    <span class="sr-only">{checked ? t('On') : t('Off')}</span>
  </button>
</div>

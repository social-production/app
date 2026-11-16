<script lang="ts">
  import type { Snippet } from 'svelte';

  import OverlaySheet from '#lib/components/overlay/OverlaySheet.svelte';

  import SettingRow from './SettingRow.svelte';

  /** A setting that needs more than a list of choices: a row showing its value, and a small
   * sheet holding its controls (a field, a key to copy, a confirmation). */
  interface Props {
    label: string;
    value: string;
    /** One line under the sheet's title, saying what the setting does. */
    hint?: string;
    /** The value is a key: shown whole, in small type. */
    key?: boolean;
    open?: boolean;
    children?: Snippet;
    footer?: Snippet;
  }

  let {
    label,
    value,
    hint = '',
    key = false,
    open = $bindable(false),
    children,
    footer,
  }: Props = $props();
</script>

<SettingRow {label} {value} {key} onOpen={() => (open = true)} />

<OverlaySheet
  {open}
  title={label}
  labelledById={`setting-${label.toLowerCase().replace(/\W+/g, '-')}`}
  rootClass="setting-sheet"
  onClose={() => (open = false)}
  {footer}
>
  {#if hint}<p class="setting-hint">{hint}</p>{/if}
  {#if children}<div class="setting-field">{@render children()}</div>{/if}
</OverlaySheet>

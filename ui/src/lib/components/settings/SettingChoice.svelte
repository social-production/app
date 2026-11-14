<script lang="ts" generics="V extends string">
  import OverlaySheet from '#lib/components/overlay/OverlaySheet.svelte';

  import SettingRow from './SettingRow.svelte';

  /** A setting with a few choices: a row showing the current one, and a small sheet listing
   * them (a coloured dot beside each when it has a `swatch`). Choosing applies at once. */
  interface Props {
    label: string;
    /** One line under the sheet's title, saying what the setting does. */
    hint?: string;
    value: V;
    options: { value: V; label: string; swatch?: string }[];
    onChange: (value: V) => void;
  }

  let { label, hint = '', value, options, onChange }: Props = $props();

  let open = $state(false);
  const current = $derived(options.find((o) => o.value === value)?.label ?? '');
</script>

<SettingRow {label} value={current} onOpen={() => (open = true)} />

<OverlaySheet
  {open}
  title={label}
  labelledById={`setting-${label.toLowerCase().replace(/\W+/g, '-')}`}
  rootClass="setting-sheet"
  onClose={() => (open = false)}
>
  {#if hint}<p class="setting-hint">{hint}</p>{/if}
  <div class="setting-options" role="radiogroup" aria-label={label}>
    {#each options as option (option.value)}
      <button
        role="radio"
        aria-checked={option.value === value}
        class="setting-option"
        class:chosen={option.value === value}
        type="button"
        onclick={() => {
          onChange(option.value);
          open = false;
        }}
      >
        {#if option.swatch}
          <span class="setting-swatch" style={`background: ${option.swatch}`}></span>
        {/if}
        <span>{option.label}</span>
        {#if option.value === value}
          <svg aria-hidden="true" viewBox="0 0 24 24" width="18" height="18">
            <path
              d="M5 12.5 10 17.5 19 7.5"
              fill="none"
              stroke="currentColor"
              stroke-width="2.2"
              stroke-linecap="round"
              stroke-linejoin="round"
            />
          </svg>
        {/if}
      </button>
    {/each}
  </div>
</OverlaySheet>

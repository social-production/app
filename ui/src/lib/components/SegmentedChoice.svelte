<script lang="ts" generics="V extends string">
  /**
   * One choice out of a few, as a soft segmented pill (ADR 0102): the UX pass's replacement
   * for an icon menu when the options fit on one line.
   */
  interface Props {
    value: V;
    ariaLabel: string;
    options: { value: V; label: string }[];
    onChange: (value: V) => void;
  }

  let { value, ariaLabel, options, onChange }: Props = $props();
</script>

<div class="segmented" role="radiogroup" aria-label={ariaLabel}>
  {#each options as option (option.value)}
    <button
      type="button"
      role="radio"
      aria-checked={option.value === value}
      class:on={option.value === value}
      onclick={() => onChange(option.value)}>{option.label}</button
    >
  {/each}
</div>

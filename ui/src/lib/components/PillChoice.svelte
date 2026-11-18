<script lang="ts" generics="V extends string">
  import { t } from '#lib/i18n/index.ts';
  /**
   * One choice out of several, as soft pills that wrap whole (Polish Pass mockups, 2026-11-16):
   * for choices too many or too long for a segmented control's one line.
   */
  interface Props {
    value: V;
    ariaLabel: string;
    options: { value: V; label: string }[];
    onChange: (value: V) => void;
  }

  let { value, ariaLabel, options, onChange }: Props = $props();
</script>

<div class="pill-choice" role="radiogroup" aria-label={ariaLabel}>
  {#each options as option (option.value)}
    <button
      type="button"
      role="radio"
      aria-checked={option.value === value}
      class:on={option.value === value}
      onclick={() => onChange(option.value)}>{t(option.label)}</button
    >
  {/each}
</div>

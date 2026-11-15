<script lang="ts">
  import '#lib/components/styles/values.css';

  /** A 1–10 rating as ten round targets you can hit with a thumb, filled up to your choice
   * (Plan, Phase and Feedback Screens, 2). The community average is shown apart. */
  interface Props {
    options: { value: number; label: string }[];
    selectedValue?: number;
    disabled?: boolean;
    leftLabel?: string;
    rightLabel?: string;
    onSelect: (value: number) => void;
  }

  let {
    options,
    selectedValue = 0,
    disabled = false,
    leftLabel = '',
    rightLabel = '',
    onSelect,
  }: Props = $props();
</script>

<div class="scale dots-scale">
  <div
    aria-label={`${leftLabel} to ${rightLabel}. Tap a number from 1 to 10.`}
    class="dots"
    role="group"
  >
    {#each options as option (option.value)}
      <button
        aria-label={option.label}
        aria-pressed={selectedValue === option.value}
        class="dot"
        class:filled={selectedValue > option.value}
        class:selected={selectedValue === option.value}
        {disabled}
        title={option.label}
        type="button"
        onclick={() => onSelect(option.value)}
      >
        {option.value}
      </button>
    {/each}
  </div>
  {#if leftLabel || rightLabel}
    <div class="ends">
      <span>{leftLabel}</span>
      <span>{rightLabel}</span>
    </div>
  {/if}
</div>

<script lang="ts">
  import '#lib/components/styles/values.css';

  /** web's DiscreteScale: a 1–10 track; the community average fills it, your vote is solid. */
  interface Props {
    options: { value: number; label: string }[];
    selectedValue?: number;
    averageValue?: number;
    disabled?: boolean;
    leftLabel?: string;
    rightLabel?: string;
    onSelect: (value: number) => void;
  }

  let {
    options,
    selectedValue = 0,
    averageValue = 0,
    disabled = false,
    leftLabel = '',
    rightLabel = '',
    onSelect,
  }: Props = $props();

  const hasUserVote = $derived(selectedValue > 0);
  const roundedAverage = $derived(averageValue > 0 ? Math.round(averageValue) : 0);
</script>

<div class="scale">
  <div
    aria-label={`${leftLabel} to ${rightLabel}. Tap a number from 1 to 10.`}
    class="track"
    role="group"
  >
    {#each options as option (option.value)}
      <button
        aria-label={option.label}
        aria-pressed={hasUserVote && selectedValue === option.value}
        class:average={roundedAverage === option.value && selectedValue !== option.value}
        class:filled={roundedAverage > 0 &&
          roundedAverage >= option.value &&
          selectedValue !== option.value}
        class:selected={hasUserVote && selectedValue === option.value}
        class="notch"
        {disabled}
        title={option.label}
        type="button"
        onclick={() => onSelect(option.value)}
      >
        {option.value}
      </button>
    {/each}
  </div>

  <div class="labels">
    <span>{leftLabel}</span>
    <span>{rightLabel}</span>
  </div>
</div>

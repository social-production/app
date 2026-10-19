<script lang="ts" generics="V extends string">
  /** web's CreateTypeSelector (without the detail panel, as the project flow uses it). */
  interface Props {
    label: string;
    name: string;
    selected: V;
    options: { value: V; label: string; summary: string }[];
    onSelect: (value: V) => void;
  }

  let { label, name, selected, options, onSelect }: Props = $props();
</script>

<div class="type-selector">
  <span class="type-selector-label" id="{name}-label">{label}</span>
  <div class="type-options" role="radiogroup" aria-labelledby="{name}-label">
    {#each options as option (option.value)}
      <button
        type="button"
        class="type-option"
        class:active={selected === option.value}
        role="radio"
        aria-checked={selected === option.value}
        onclick={() => onSelect(option.value)}
      >
        <span class="type-option-title">{option.label}</span>
        <span class="type-option-summary">{option.summary}</span>
      </button>
    {/each}
  </div>
</div>

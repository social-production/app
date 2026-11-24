<script lang="ts" generics="V extends string">
  import type { Snippet } from 'svelte';

  import '#lib/components/styles/ux/choicerows.css';

  /**
   * One choice out of a few as tappable rows (Edit, Plan and Meetup mockups, owner 2026-11-24:
   * the In person / Online switcher "looks like the old way"): an icon, a title and a short
   * line each; the chosen row is tinted and can open its own fields under it.
   */
  interface Option {
    value: V;
    label: string;
    hint?: string | undefined;
    /** A path on a 24×24 grid, drawn with a round stroke. */
    icon?: string | undefined;
  }

  interface Props {
    value: V;
    ariaLabel: string;
    options: Option[];
    onChange: (value: V) => void;
    /** Fields for the chosen row, shown under it. */
    under?: Snippet<[V]>;
  }

  let { value, ariaLabel, options, onChange, under }: Props = $props();
</script>

<div class="choice-rows" role="radiogroup" aria-label={ariaLabel}>
  {#each options as option (option.value)}
    {@const on = option.value === value}
    <button
      type="button"
      role="radio"
      class="choice-row"
      class:on
      aria-checked={on}
      onclick={() => onChange(option.value)}
    >
      {#if option.icon}
        <span class="choice-icon" aria-hidden="true"
          ><svg viewBox="0 0 24 24"><path d={option.icon} /></svg></span
        >
      {/if}
      <span class="choice-copy">
        <b>{option.label}</b>
        {#if option.hint}<small>{option.hint}</small>{/if}
      </span>
      <span class="choice-mark" aria-hidden="true"></span>
    </button>
    {#if on && under}<div class="choice-under">{@render under(option.value)}</div>{/if}
  {/each}
</div>

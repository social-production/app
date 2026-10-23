<script lang="ts">
  /** web's MentionMenu: names matching the "@…" being typed, above the composer. */
  interface Props {
    results: { agent: string; name: string }[];
    onSelect: (name: string) => void;
  }

  let { results, onSelect }: Props = $props();
</script>

{#if results.length > 0}
  <div class="mention-menu" role="listbox">
    {#each results as person (person.agent)}
      <button
        class="mention-item"
        role="option"
        aria-selected="false"
        type="button"
        onmousedown={(event) => event.preventDefault()}
        onclick={() => onSelect(person.name)}>{person.name}</button
      >
    {/each}
  </div>
{/if}

<script lang="ts">
  import type { Mentionable } from '#lib/domain/mentions.ts';
  import { keyTag } from '#lib/domain/people.ts';

  /**
   * web's MentionMenu: names matching the "@…" being typed, above the composer, nearest first.
   * Names aren't unique here, so a name shown twice carries its key tag.
   */
  interface Props {
    results: Mentionable[];
    onSelect: (name: string) => void;
  }

  let { results, onSelect }: Props = $props();

  const shared = $derived(
    new Set(
      results.map((p) => p.name.toLowerCase()).filter((name, i, all) => all.indexOf(name) !== i),
    ),
  );
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
        onclick={() => onSelect(person.name)}
        >{person.name}{#if shared.has(person.name.toLowerCase())}<span class="mention-tag"
            >· {keyTag(person.agent)}</span
          >{/if}</button
      >
    {/each}
  </div>
{/if}

<script lang="ts">
  import { TAG_MAX, TAGS_MAX } from '#lib/domain/projects.ts';

  /**
   * web's CreateScopeTagSelector, for free tags until channels and communities arrive (M5).
   * Enter or a comma adds the typed tag; a chip removes itself.
   */
  interface Props {
    label: string;
    tags: string[];
    onChange: (tags: string[]) => void;
  }

  let { label, tags, onChange }: Props = $props();

  let query = $state('');

  function add(): void {
    const tag = query.trim().replace(/,$/, '').slice(0, TAG_MAX);
    query = '';
    if (tag && !tags.includes(tag) && tags.length < TAGS_MAX) onChange([...tags, tag]);
  }

  function key(event: KeyboardEvent): void {
    if (event.key === 'Enter' || event.key === ',') {
      event.preventDefault();
      add();
    }
  }
</script>

<div>
  <span class="field-label">{label}</span>
  <div class="token-input-stack">
    {#if tags.length > 0}
      <div class="selected-row" aria-label="Selected tags">
        {#each tags as tag (tag)}
          <button
            class="selected-chip"
            type="button"
            onclick={() => onChange(tags.filter((t) => t !== tag))}
          >
            <span>{tag}</span>
            <span class="chip-remove" aria-hidden="true">×</span>
          </button>
        {/each}
      </div>
    {/if}
    <input bind:value={query} placeholder="Type a tag, then Enter" onkeydown={key} onblur={add} />
  </div>
  <p class="helper-text">Up to {TAGS_MAX} tags. People find projects by them in search.</p>
</div>

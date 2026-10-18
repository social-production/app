<script lang="ts">
  let { words }: { words: string[] } = $props();
</script>

<ol class="grid">
  {#each words as word, i (i)}
    <li><span class="n">{i + 1}</span> {word}</li>
  {/each}
</ol>

<style>
  .grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(120px, 1fr));
    gap: 6px;
    margin: 0;
    padding: 0;
    list-style: none;
    font-family: ui-monospace, monospace;
  }
  li {
    padding: 6px 8px;
    border-radius: var(--radius-sm);
    background: var(--panel-soft);
  }
  .n {
    color: var(--text-soft);
    margin-right: 4px;
  }
</style>

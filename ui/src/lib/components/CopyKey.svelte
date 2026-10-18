<script lang="ts">
  let { value, label = 'Copy my key' }: { value: string; label?: string } = $props();

  let copied = $state(false);

  async function copy(): Promise<void> {
    try {
      await navigator.clipboard.writeText(value);
      copied = true;
      setTimeout(() => (copied = false), 2000);
    } catch {
      copied = false; // clipboard blocked: the key is shown below to select by hand
    }
  }
</script>

<div class="copy-key">
  <button class="button-secondary" type="button" onclick={copy}>{copied ? 'Copied!' : label}</button
  >
  <code class="key">{value}</code>
</div>

<style>
  .copy-key {
    display: grid;
    gap: 6px;
    justify-items: start;
    min-width: 0;
  }
  .key {
    max-width: 100%;
    color: var(--text-soft);
    font-size: 12px;
    word-break: break-all;
  }
</style>

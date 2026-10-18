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

<div class="row">
  <button onclick={copy}>{copied ? 'Copied!' : label}</button>
  <span class="muted key">{value}</span>
</div>

<style>
  .key {
    font-family: ui-monospace, monospace;
    font-size: 0.8rem;
    word-break: break-all;
  }
</style>

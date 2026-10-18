<script lang="ts">
  interface Props {
    tone: 'ok' | 'pending' | 'error';
    label: string;
  }

  let { tone, label }: Props = $props();
</script>

<span class="pill {tone}">{label}</span>

<style>
  .pill {
    display: inline-block;
    padding: 2px 10px;
    border-radius: var(--radius-lg);
    font-size: 0.85rem;
    background: var(--panel-strong);
    color: var(--text-soft);
  }
  .ok {
    background: var(--brand-soft);
    color: var(--brand-strong);
  }
  .pending {
    background: var(--status-yellow-soft);
    color: var(--status-yellow-strong);
  }
  .error {
    background: var(--need-soft);
    color: var(--need-strong);
  }
</style>

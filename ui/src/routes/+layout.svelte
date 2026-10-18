<script lang="ts">
  import type { Snippet } from 'svelte';

  import AppShell from '#lib/features/shell/AppShell.svelte';

  import '../app.css';

  let { children }: { children: Snippet } = $props();
</script>

<AppShell>{@render children()}</AppShell>

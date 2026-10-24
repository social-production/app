<script lang="ts">
  import type { Snippet } from 'svelte';

  import CallHost from '#lib/features/calls/CallHost.svelte';
  import AppShell from '#lib/features/shell/AppShell.svelte';

  import '../app.css';

  let { children }: { children: Snippet } = $props();
</script>

<!-- Pages that need an account wrap themselves in AccountGate; the feed and about are public. -->
<AppShell>{@render children()}</AppShell>
<CallHost />

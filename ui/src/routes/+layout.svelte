<script lang="ts">
  import type { Snippet } from 'svelte';

  import CallHost from '#lib/features/calls/CallHost.svelte';
  import MapHost from '#lib/features/map/MapHost.svelte';
  import AppShell from '#lib/features/shell/AppShell.svelte';

  import '../app.css';

  let { children }: { children: Snippet } = $props();
</script>

<!-- Pages that need an account wrap themselves in AccountGate; the feed and about are public. -->
<AppShell>
  {@render children()}
  {#snippet mapPanel(close: () => void)}<MapHost onClose={close} />{/snippet}
</AppShell>
<CallHost />

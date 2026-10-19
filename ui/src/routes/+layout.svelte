<script lang="ts">
  import type { Snippet } from 'svelte';

  import AccountGate from '#lib/features/account/AccountGate.svelte';
  import AppShell from '#lib/features/shell/AppShell.svelte';

  import '../app.css';

  let { children }: { children: Snippet } = $props();
</script>

<AppShell>
  <AccountGate>{@render children()}</AccountGate>
</AppShell>

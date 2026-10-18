<script lang="ts">
  import type { Snippet } from 'svelte';

  import { resolve } from '$app/paths';

  import StatusPill from '#lib/components/StatusPill.svelte';
  import AccountGate from '#lib/features/account/AccountGate.svelte';
  import { signOut } from '#lib/services/account.ts';
  import { connection } from '#lib/stores/connection.svelte.ts';

  let { children }: { children: Snippet } = $props();
</script>

<header class="bar">
  <strong>Social Production</strong>
  {#if connection.current.status === 'connected'}
    <nav class="row">
      <a href={resolve('/')}>People</a>
      <a href={resolve('/profile')}>Profile</a>
      <a href={resolve('/health')}>Network</a>
    </nav>
    <StatusPill tone="ok" label={connection.current.node} />
    <button onclick={() => signOut()}>Sign out</button>
  {/if}
</header>

<main>
  <AccountGate>{@render children()}</AccountGate>
</main>

<style>
  .bar {
    display: flex;
    gap: 16px;
    align-items: center;
    flex-wrap: wrap;
    padding: 12px var(--page-gutter);
    background: var(--toolbar-background);
    border-bottom: 1px solid var(--panel-border);
  }
  nav {
    flex: 1;
  }
  main {
    max-width: 900px;
    margin: 0 auto;
    padding: var(--page-gutter);
  }
</style>

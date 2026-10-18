<script lang="ts">
  import type { Snippet } from 'svelte';

  import { connect, signOut } from '#lib/services/account.ts';
  import { connection } from '#lib/stores/connection.svelte.ts';

  import Welcome from './Welcome.svelte';

  let { children }: { children: Snippet } = $props();

  $effect(() => {
    void connect();
  });
</script>

{#if connection.current.status === 'connected'}
  {@render children()}
{:else if connection.current.status === 'signed-out'}
  <Welcome />
{:else if connection.current.status === 'offline'}
  <section class="panel stack narrow">
    <h2>Can't reach the network</h2>
    <p class="error">{connection.current.reason}</p>
    <div class="row">
      <button onclick={() => connect()}>Try again</button>
      <button onclick={() => signOut()}>Sign in again</button>
    </div>
  </section>
{:else}
  <p class="muted narrow">Connecting…</p>
{/if}

<style>
  .narrow {
    max-width: 560px;
    margin: 24px auto;
  }
</style>

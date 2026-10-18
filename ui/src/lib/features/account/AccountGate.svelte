<script lang="ts">
  import type { Snippet } from 'svelte';

  import { connect, signOut } from '#lib/services/account.ts';
  import { connection } from '#lib/stores/connection.svelte.ts';

  import Welcome from './Welcome.svelte';

  import '#lib/components/styles/onboarding.css';

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
  <div class="onboarding">
    <section class="page">
      <section class="hero-card">
        <h1>Can't reach the network</h1>
        <p class="status-note">{connection.current.reason}</p>
        <div class="button-row">
          <button class="button-primary" type="button" onclick={() => connect()}>Try again</button>
          <button class="button-secondary" type="button" onclick={() => signOut()}
            >Sign in again</button
          >
        </div>
      </section>
    </section>
  </div>
{:else}
  <p class="connecting">Connecting…</p>
{/if}

<style>
  .connecting {
    padding: 24px var(--page-gutter);
    color: var(--text-soft);
  }
</style>

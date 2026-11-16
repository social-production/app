<script lang="ts">
  import { connection } from '#lib/stores/connection.svelte.ts';

  import { goto } from '$app/navigation';
  import { resolve } from '$app/paths';

  import OpeningScreen from '#lib/components/loading/OpeningScreen.svelte';

  import Welcome from './Welcome.svelte';

  import '#lib/components/styles/onboarding.css';

  /** web's /onboarding: sign up or log in; once signed in, back to the feed. */
  $effect(() => {
    if (connection.current.status === 'connected') void goto(resolve('/'));
  });
</script>

{#if connection.current.status === 'signed-out' || connection.current.status === 'offline'}
  <Welcome />
{:else}
  <OpeningScreen />
{/if}

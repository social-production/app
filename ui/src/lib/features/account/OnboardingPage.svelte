<script lang="ts">
  import { connection } from '#lib/stores/connection.svelte.ts';

  import { goto } from '$app/navigation';
  import { resolve } from '$app/paths';
  import { page } from '$app/state';

  import OpeningScreen from '#lib/components/loading/OpeningScreen.svelte';

  import Welcome from './Welcome.svelte';

  import '#lib/components/styles/onboarding.css';

  /** web's /onboarding: sign up or log in; once signed in, back to the page that sent you
   * (`?next=`, a path on this site only), else the feed. */
  const back = (): string => {
    const next = page.url.searchParams.get('next') ?? '';
    return next.startsWith('/') && !next.startsWith('//') ? next : resolve('/');
  };
  $effect(() => {
    // eslint-disable-next-line svelte/no-navigation-without-resolve -- a path on this site
    if (connection.current.status === 'connected') void goto(back(), { replaceState: true });
  });
</script>

{#if connection.current.status === 'signed-out' || connection.current.status === 'offline'}
  <Welcome />
{:else}
  <OpeningScreen />
{/if}

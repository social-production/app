<script lang="ts">
  import type { Snippet } from 'svelte';

  import { connection } from '#lib/stores/connection.svelte.ts';

  import ProfileView from './ProfileView.svelte';

  let { children }: { children?: Snippet } = $props();
</script>

{#if connection.current.status === 'connected'}
  <ProfileView agent={connection.current.agent} {children} />
{/if}

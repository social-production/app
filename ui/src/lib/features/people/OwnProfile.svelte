<script lang="ts">
  import { connection } from '#lib/stores/connection.svelte.ts';

  import ProfileView from './ProfileView.svelte';
</script>

{#if connection.current.status === 'connected'}
  <ProfileView agent={connection.current.agent} />
{/if}

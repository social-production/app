<script lang="ts">
  import { page } from '$app/state';

  import ProfileView from '#lib/features/people/ProfileView.svelte';
</script>

{#key page.params.agent}
  <ProfileView agent={page.params.agent ?? ''} />
{/key}

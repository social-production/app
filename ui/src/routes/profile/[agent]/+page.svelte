<script lang="ts">
  import AccountGate from '#lib/features/account/AccountGate.svelte';
  import ProfileView from '#lib/features/people/ProfileView.svelte';
  import ProfileFeed from '#lib/features/projects/ProfileFeed.svelte';

  import { page } from '$app/state';
</script>

<AccountGate guests>
  {#key page.params.agent}
    <ProfileView agent={page.params.agent ?? ''}>
      <ProfileFeed agent={page.params.agent ?? ''} />
    </ProfileView>
  {/key}
</AccountGate>

<script lang="ts">
  import AccountGate from '#lib/features/account/AccountGate.svelte';
  import { page } from '$app/state';

  import ProfileView from '#lib/features/people/ProfileView.svelte';
</script>

<AccountGate>
  {#key page.params.agent}
    <ProfileView agent={page.params.agent ?? ''} />
  {/key}
</AccountGate>

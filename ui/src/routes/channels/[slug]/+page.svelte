<script lang="ts">
  import AccountGate from '#lib/features/account/AccountGate.svelte';
  import ScopePage from '#lib/features/projects/scopes/ScopePage.svelte';

  import { page } from '$app/state';
</script>

<AccountGate guests>
  {#key page.params.slug}
    <ScopePage kind="channel" slug={page.params.slug ?? ''} />
  {/key}
</AccountGate>

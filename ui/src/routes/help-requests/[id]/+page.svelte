<script lang="ts">
  import AccountGate from '#lib/features/account/AccountGate.svelte';
  import HelpRequestDetail from '#lib/features/help-requests/HelpRequestDetail.svelte';

  import { page } from '$app/state';
</script>

<AccountGate guests>
  {#key page.params.id}
    <HelpRequestDetail id={page.params.id ?? ''} />
  {/key}
</AccountGate>

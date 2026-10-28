<script lang="ts">
  import AccountGate from '#lib/features/account/AccountGate.svelte';
  import HelpRequestDetail from '#lib/features/projects/help/HelpRequestDetail.svelte';

  import { page } from '$app/state';
</script>

<AccountGate guests>
  {#key page.params.id}
    <HelpRequestDetail id={page.params.id ?? ''} />
  {/key}
</AccountGate>

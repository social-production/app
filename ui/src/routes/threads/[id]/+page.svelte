<script lang="ts">
  import AccountGate from '#lib/features/account/AccountGate.svelte';
  import DiscussionPage from '#lib/features/projects/discussions/DiscussionPage.svelte';

  import { page } from '$app/state';
</script>

<AccountGate guests>
  {#key page.params.id}
    <DiscussionPage id={page.params.id ?? ''} kind="thread" />
  {/key}
</AccountGate>

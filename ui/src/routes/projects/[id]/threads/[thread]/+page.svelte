<script lang="ts">
  import AccountGate from '#lib/features/account/AccountGate.svelte';
  import ThreadPage from '#lib/features/threads/ThreadPage.svelte';

  import { page } from '$app/state';
</script>

<AccountGate>
  {#key `${page.params.id}/${page.params.thread}`}
    <ThreadPage projectId={page.params.id ?? ''} threadId={page.params.thread ?? ''} />
  {/key}
</AccountGate>

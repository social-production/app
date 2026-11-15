<script lang="ts">
  import AccountGate from '#lib/features/account/AccountGate.svelte';
  import FeedbackItemPage from '#lib/features/feedback/FeedbackItemPage.svelte';

  import { page } from '$app/state';
</script>

<AccountGate guests>
  {#key page.params.id}
    <FeedbackItemPage id={page.params.id ?? ''} />
  {/key}
</AccountGate>

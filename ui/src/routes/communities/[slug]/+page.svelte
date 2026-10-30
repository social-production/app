<script lang="ts">
  import AccountGate from '#lib/features/account/AccountGate.svelte';
  import ScopePage from '#lib/features/projects/scopes/ScopePage.svelte';

  import { page } from '$app/state';
</script>

<AccountGate guests>
  {#key `${page.params.slug}${page.url.hash}`}
    <ScopePage kind="community" slug={page.params.slug ?? ''} hash={page.url.hash} />
  {/key}
</AccountGate>

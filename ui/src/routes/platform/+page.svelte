<script lang="ts">
  import AccountGate from '#lib/features/account/AccountGate.svelte';
  import ScopePage from '#lib/features/projects/scopes/ScopePage.svelte';

  import { page } from '$app/state';
</script>

<!-- web's /platform: the Platform channel's scope page; `?board=1` opens the moderators. -->
<AccountGate guests>
  <ScopePage kind="channel" slug="platform" board={page.url.searchParams.get('board') === '1'} />
</AccountGate>

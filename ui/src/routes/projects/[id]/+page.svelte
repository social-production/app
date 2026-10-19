<script lang="ts">
  import AccountGate from '#lib/features/account/AccountGate.svelte';
  import ProjectDetail from '#lib/features/projects/ProjectDetail.svelte';

  import { page } from '$app/state';
</script>

<AccountGate>
  {#key page.params.id}
    <ProjectDetail id={page.params.id ?? ''} />
  {/key}
</AccountGate>

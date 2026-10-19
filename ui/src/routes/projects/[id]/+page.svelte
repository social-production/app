<script lang="ts">
  import AccountGate from '#lib/features/account/AccountGate.svelte';
  import ProjectDetail from '#lib/features/projects/ProjectDetail.svelte';
  import ProjectThreads from '#lib/features/threads/ProjectThreads.svelte';

  import { page } from '$app/state';
</script>

<AccountGate>
  {#key page.params.id}
    <ProjectDetail id={page.params.id ?? ''}>
      {#snippet threads()}
        <ProjectThreads projectId={page.params.id ?? ''} />
      {/snippet}
    </ProjectDetail>
  {/key}
</AccountGate>

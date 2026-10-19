<script lang="ts">
  import ProjectDetail from '#lib/features/projects/ProjectDetail.svelte';

  import { page } from '$app/state';
</script>

{#key page.params.id}
  <ProjectDetail id={page.params.id ?? ''} />
{/key}

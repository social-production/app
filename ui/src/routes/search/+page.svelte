<script lang="ts">
  import ProjectFeed from '#lib/features/projects/ProjectFeed.svelte';

  import { page } from '$app/state';
</script>

<ProjectFeed search={page.url.searchParams.get('q') ?? ''} />

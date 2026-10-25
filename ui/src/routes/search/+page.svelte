<script lang="ts">
  import SearchPage from '#lib/features/search/SearchPage.svelte';

  import { page } from '$app/state';
</script>

<SearchPage
  query={page.url.searchParams.get('q') ?? ''}
  typeParam={page.url.searchParams.get('type')}
/>

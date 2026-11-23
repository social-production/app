<script lang="ts">
  import type { Snippet } from 'svelte';

  import CallHost from '#lib/features/calls/CallHost.svelte';
  import MapHost from '#lib/features/map/MapHost.svelte';
  import SearchOverlay from '#lib/features/search/SearchOverlay.svelte';
  import AppShell from '#lib/features/shell/AppShell.svelte';
  import PhotoCropHost from '#lib/features/shell/PhotoCropHost.svelte';
  import { resumePlace } from '#lib/features/shell/resumePlace.ts';
  import { followViewportTop } from '#lib/features/shell/viewportTop.ts';

  import '../app.css';

  let { children }: { children: Snippet } = $props();

  // A phone that threw the app away in the background reopens it where you were.
  $effect(() => resumePlace());
  // iPhone: keeps a chat's top tabs in sight while the keyboard slides the page.
  $effect(() => followViewportTop());
</script>

<!-- Pages that need an account wrap themselves in AccountGate; the feed and about are public. -->
<AppShell>
  {@render children()}
  {#snippet mapPanel(close: () => void)}<MapHost onClose={close} />{/snippet}
</AppShell>
<CallHost />
<PhotoCropHost />
<SearchOverlay />

<script lang="ts">
  import { goto } from '$app/navigation';
  import { resolve } from '$app/paths';

  import MapHost from '#lib/features/map/MapHost.svelte';

  // web's `/map` page: the panel fills the frame; leaving it goes back to the feed.
  $effect(() => {
    document.documentElement.style.overflow = 'hidden';
    document.body.style.overflow = 'hidden';
    return () => {
      document.documentElement.style.overflow = '';
      document.body.style.overflow = '';
    };
  });
</script>

<section class="map-page">
  <MapHost embedded={false} onClose={() => void goto(resolve('/'))} />
</section>

<style>
  .map-page {
    display: flex;
    flex-direction: column;
    gap: 12px;
    height: 100%;
    min-height: 0;
    overflow: hidden;
    overscroll-behavior: none;
  }

  .map-page :global(.map-panel) {
    flex: 1 1 auto;
    min-height: 0;
    height: 100%;
  }
</style>

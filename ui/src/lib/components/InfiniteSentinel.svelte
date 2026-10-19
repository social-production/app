<script lang="ts">
  /** web's InfiniteFeedSentinel: asks for more when it scrolls within 240px of view. */
  let { disabled, onMore }: { disabled: boolean; onMore: () => void } = $props();

  let sentinel = $state<HTMLDivElement>();

  $effect(() => {
    if (!sentinel || disabled) return;
    const observer = new IntersectionObserver(
      (entries) => {
        if (entries.some((e) => e.isIntersecting)) onMore();
      },
      { rootMargin: '240px 0px' },
    );
    observer.observe(sentinel);
    return () => observer.disconnect();
  });
</script>

<div bind:this={sentinel} aria-hidden="true" class="feed-sentinel"></div>

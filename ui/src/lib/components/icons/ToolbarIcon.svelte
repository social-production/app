<script lang="ts">
  import { toolbarIcons } from './toolbarIcons.ts';

  /** Same icons and sizes as web's FeedToolbarIcon. */
  let { name }: { name: string } = $props();
</script>

<svg aria-hidden="true" viewBox="0 0 24 24" class="toolbar-icon">
  <!-- eslint-disable-next-line svelte/no-at-html-tags -- static, trusted markup from toolbarIcons.ts -->
  {@html toolbarIcons[name] ?? ''}
</svg>

<style>
  .toolbar-icon {
    width: 18px;
    height: 18px;
    display: block;
  }
</style>

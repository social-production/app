<script lang="ts">
  import { panelIcons } from './panelIcons.ts';

  let { name, className }: { name: keyof typeof panelIcons; className: string } = $props();
</script>

<svg aria-hidden="true" viewBox="0 0 24 24" class={className}>
  <!-- eslint-disable-next-line svelte/no-at-html-tags -- static, trusted markup from panelIcons.ts -->
  {@html panelIcons[name]}
</svg>

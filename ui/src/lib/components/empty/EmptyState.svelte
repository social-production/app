<script lang="ts">
  import type { Snippet } from 'svelte';

  import ToolbarIcon from '#lib/components/icons/ToolbarIcon.svelte';

  /**
   * An empty feed or list (Scope Filters mockups §2): a soft tile with the place's icon, one
   * bold line saying what is missing, one sentence saying what fills it, and at most one
   * action (`children`, a `.empty-action` link or button).
   */
  interface Props {
    icon: string;
    title: string;
    text?: string;
    children?: Snippet;
  }

  let { icon, title, text = '', children }: Props = $props();
</script>

<section class="empty-block">
  <span class="empty-tile" aria-hidden="true"><ToolbarIcon name={icon} /></span>
  <b>{title}</b>
  {#if text}<p>{text}</p>{/if}
  {#if children}{@render children()}{/if}
</section>

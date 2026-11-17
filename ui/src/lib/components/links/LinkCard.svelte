<script lang="ts">
  import type { Snippet } from 'svelte';

  import { resolve } from '$app/paths';

  import SurfaceIcon from '#lib/components/icons/SurfaceIcon.svelte';
  import type { SurfaceIconId } from '#lib/components/icons/surface.ts';
  import ToolbarIcon from '#lib/components/icons/ToolbarIcon.svelte';

  import '#lib/components/styles/links.css';

  /**
   * A linked (or past) project as one row (Polish Pass mockups, 2026-11-16): its glyph, title
   * and one line, with a status tag for a past one. Tapping opens its details below, with Open
   * and the row's own action.
   */
  interface Props {
    title: string;
    metaLine: string;
    /** A line that marks a vote running on this link (e.g. to end it). */
    voteLine?: string | null;
    outcomeLabel?: string | null;
    outcomeTone?: 'approved' | 'blocked' | 'pending' | null;
    /** The counterpart's project page. */
    projectId: string;
    /** The row's glyph: the project's kind, or a round link for a past one (Polish Pass §6). */
    icon?: SurfaceIconId | 'link';
    expanded: boolean;
    domId: string;
    onToggle: () => void;
    children: Snippet;
  }

  let {
    title,
    metaLine,
    voteLine = null,
    outcomeLabel = null,
    outcomeTone = null,
    projectId,
    icon = 'cube',
    expanded,
    domId,
    onToggle,
    children,
  }: Props = $props();

  const detailsId = $derived(`${domId}-details`);
</script>

<article id={domId} class="link-card" class:expanded>
  <button
    class="toggle-button"
    type="button"
    aria-expanded={expanded}
    aria-controls={detailsId}
    onclick={onToggle}
  >
    <span class="identity" class:round={icon === 'link'} aria-hidden="true">
      {#if icon === 'link'}<ToolbarIcon name="link" />{:else}<SurfaceIcon {icon} size="sm" />{/if}
    </span>
    <span class="link-row-copy">
      <strong class="card-title">{title}</strong>
      <span class="row-meta">{metaLine}</span>
      {#if voteLine}<span class="side-vote-line">{voteLine}</span>{/if}
    </span>
    {#if outcomeLabel}
      <span class={`outcome-label ${outcomeTone ?? 'pending'}`}>{outcomeLabel}</span>
    {:else}
      <svg class="link-chevron" aria-hidden="true" viewBox="0 0 24 24"
        ><path d="m7 10 5 5 5-5" /></svg
      >
    {/if}
  </button>
  {#if expanded}
    <div id={detailsId} class="card-body">
      {@render children()}
      <a class="open-button" href={resolve('/projects/[id]', { id: projectId })}>Open {title}</a>
    </div>
  {/if}
</article>

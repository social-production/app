<script lang="ts">
  import type { Snippet } from 'svelte';

  import { resolve } from '$app/paths';

  import SurfaceIcon from '#lib/components/icons/SurfaceIcon.svelte';

  import '#lib/components/styles/links.css';

  /** web's LinkCard: the counterpart's title and vote lines; tapping opens its details. */
  interface Props {
    title: string;
    metaLine: string;
    sideVoteLines?: string[];
    outcomeLabel?: string | null;
    outcomeTone?: 'approved' | 'blocked' | 'pending' | null;
    accent: string;
    /** The counterpart's project page. */
    projectId: string;
    expanded: boolean;
    domId: string;
    onToggle: () => void;
    children: Snippet;
  }

  let {
    title,
    metaLine,
    sideVoteLines = [],
    outcomeLabel = null,
    outcomeTone = null,
    accent,
    projectId,
    expanded,
    domId,
    onToggle,
    children,
  }: Props = $props();

  const detailsId = $derived(`${domId}-details`);
  const hasMeta = $derived(!!metaLine || sideVoteLines.length > 0);
</script>

<article id={domId} class="link-card" class:expanded style={`--row-accent: ${accent};`}>
  <div class="card-header">
    <button
      class="toggle-button"
      type="button"
      aria-expanded={expanded}
      aria-controls={detailsId}
      onclick={onToggle}
    >
      <div class="header-top">
        <span class="identity" aria-hidden="true"><SurfaceIcon icon="project" size="sm" /></span>
        <strong class="card-title">{title}</strong>
        {#if outcomeLabel}
          <span class={`outcome-label ${outcomeTone ?? 'pending'}`}>{outcomeLabel}</span>
        {/if}
      </div>
      {#if hasMeta}
        <div class="header-meta">
          {#if metaLine}<span class="row-meta">{metaLine}</span>{/if}
          {#if sideVoteLines.length > 0}
            <div class="side-votes">
              {#each sideVoteLines as line (line)}
                <span class="side-vote-line">{line}</span>
              {/each}
            </div>
          {/if}
        </div>
      {/if}
    </button>
    <a class="open-button" href={resolve('/projects/[id]', { id: projectId })}>Open</a>
  </div>
  {#if expanded}
    <div id={detailsId} class="card-body">{@render children()}</div>
  {/if}
</article>

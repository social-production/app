<script lang="ts">
  import type { Snippet } from 'svelte';

  import FactIcon, { type FactIconName } from './FactIcon.svelte';

  /**
   * One row of a detail page's facts card (Detail Pages Cleanup §3): the icon at 16, the text
   * at 48, and a chevron when the row opens something (a link with `href`, a button with
   * `onclick`). `aside` sits at the right, before the chevron (e.g. faces).
   */
  interface Props {
    icon: FactIconName;
    title: string;
    sub?: string;
    href?: string;
    external?: boolean;
    label?: string;
    onclick?: (e: MouseEvent) => void;
    aside?: Snippet;
  }

  let { icon, title, sub = '', href, external = false, label, onclick, aside }: Props = $props();
</script>

{#snippet body()}
  <span class="fact-icon"><FactIcon name={icon} /></span>
  <span class="fact-text"
    ><strong>{title}</strong>{#if sub}<small>{sub}</small>{/if}</span
  >
  {#if aside}<span class="fact-aside">{@render aside()}</span>{/if}
  {#if href || onclick}
    <svg class="fact-chevron" aria-hidden="true" viewBox="0 0 24 24"
      ><path d="M9.5 6.5 15 12l-5.5 5.5" /></svg
    >
  {/if}
{/snippet}

{#if href}
  <!-- eslint-disable svelte/no-navigation-without-resolve -- callers pass resolved or external hrefs -->
  <a
    class="fact-row"
    {href}
    aria-label={label}
    rel={external ? 'noopener noreferrer' : undefined}
    {onclick}>{@render body()}</a
  >
  <!-- eslint-enable svelte/no-navigation-without-resolve -->
{:else if onclick}
  <button class="fact-row" type="button" aria-label={label} {onclick}>{@render body()}</button>
{:else}
  <div class="fact-row">{@render body()}</div>
{/if}

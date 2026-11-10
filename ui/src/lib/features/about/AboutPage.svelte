<script lang="ts">
  import { goto } from '$app/navigation';
  import { resolve } from '$app/paths';
  import { page } from '$app/state';

  import {
    ABOUT_SECTIONS,
    aboutDecisions,
    aboutHow,
    aboutLinks,
    aboutNetwork,
    aboutReading,
    aboutWhy,
    type AboutTabId,
    type Block,
  } from './aboutContent.ts';

  import '#lib/components/styles/about.css';

  /** The About page in web's tabbed reader, the open tab in `?section=` (owner's own copy,
   * 2026-11-10: fewer sections than web's). */
  const sectionIds = new Set<string>(ABOUT_SECTIONS.map((section) => section.id));

  function readSection(value: string | null): AboutTabId {
    return value && sectionIds.has(value) ? (value as AboutTabId) : 'why';
  }

  const active = $derived(readSection(page.url.searchParams.get('section')));
  const title = $derived(ABOUT_SECTIONS.find((s) => s.id === active)?.label ?? '');
  const blocks = $derived(
    { why: aboutWhy, how: aboutHow, decisions: aboutDecisions, network: aboutNetwork }[active],
  );

  function selectSection(id: AboutTabId): void {
    const next = new URL(page.url.href);
    if (id === 'why') next.searchParams.delete('section');
    else next.searchParams.set('section', id);
    // eslint-disable-next-line svelte/no-navigation-without-resolve -- this page's own path
    void goto(`${next.pathname}${next.search}`, { replace: true, reset: false });
  }
</script>

{#snippet section(items: Block[])}
  {#each items as block, i (i)}
    {#if block.title}<h3>{block.title}</h3>{/if}
    {#each block.paragraphs as paragraph (paragraph)}
      <p>{paragraph}</p>
    {/each}
  {/each}
{/snippet}

<div class="about-page">
  <section class="page">
    <header class="hero">
      <p class="eyebrow">About</p>
      <h1>Organising useful work together, for use rather than sale.</h1>
    </header>

    <div class="reader">
      <div class="tab-row" role="tablist" aria-label="About">
        {#each ABOUT_SECTIONS as tab (tab.id)}
          <button
            aria-selected={active === tab.id}
            class:active-tab={active === tab.id}
            class="detail-surface-tab tab"
            role="tab"
            type="button"
            onclick={() => selectSection(tab.id)}
          >
            <span class="tab-label-full">{tab.label}</span>
            <span class="tab-label-compact">{tab.short}</span>
          </button>
        {/each}
      </div>

      <article class="essay">
        <h2>{title}</h2>
        {@render section(blocks)}
        {#if active === 'why'}
          <h3>Further reading</h3>
          {#each aboutReading as link (link.href)}
            <p>
              <a href={link.href} rel="external noreferrer" target="_blank">{link.label}</a>
              {link.blurb}
            </p>
          {/each}
        {:else if active === 'network'}
          <p class="about-links">
            {#each aboutLinks as link, i (link.href)}
              {#if i > 0}·{/if}
              <a href={link.href} rel="external noreferrer" target="_blank">{link.label}</a>
            {/each}
          </p>
        {/if}

        <a class="discover" href={resolve('/')}>Get involved!</a>
      </article>
    </div>
  </section>
</div>

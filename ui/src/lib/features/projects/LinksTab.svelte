<script lang="ts">
  import LinkComposer from '#lib/components/links/LinkComposer.svelte';
  import LinkRequestsSection from '#lib/components/links/LinkRequestsSection.svelte';
  import LinksSection from '#lib/components/links/LinksSection.svelte';
  import LinkVoteHistory from '#lib/components/links/LinkVoteHistory.svelte';
  import { LINKS_INTRO } from '#lib/domain/links.ts';

  import type { DetailState } from './detailState.svelte.ts';
  import { LinksState } from './linksState.svelte.ts';

  import '#lib/components/styles/links.css';

  /**
   * web's DetailLinksTab for projects: the venue, active links, pending requests, historical
   * links and past votes. Events and conversions arrive with their own screens.
   */
  let { detail }: { detail: DetailState } = $props();

  const links = new LinksState(() => detail.project);
  let composing = $state(false);

  $effect(() => {
    void links.load();
  });

  const frame = $derived(links.frame);
  const venue = $derived(detail.project?.charter?.location_label.trim() ?? '');

  async function submit(targetId: string, summary: string): Promise<void> {
    if (await links.propose(targetId, summary)) composing = false;
  }
</script>

<section class="links-tab">
  <header class="intro">
    <h1>Links</h1>
    <p>{LINKS_INTRO}</p>
  </header>

  {#if venue && venue.toLowerCase() !== 'online'}
    <section class="external-links">
      <a
        class="external-link"
        href={`https://www.openstreetmap.org/search?query=${encodeURIComponent(venue)}`}
        target="_blank"
        rel="noopener noreferrer"
      >
        <strong>Venue</strong>
        <span>{venue}</span>
      </a>
    </section>
  {/if}

  {#if links.error}<p class="tab-loading" role="alert">{links.error}</p>{/if}

  {#if !frame}
    <p class="tab-loading">Loading links…</p>
  {:else}
    <LinksSection
      title="Active links"
      description="Approved connections to related projects and events. Active links stay voteable so members can sever them."
      items={frame.active}
      emptyMessage="No active links yet."
      allowSever={frame.canPropose}
      pendingId={links.pendingId}
      onSever={(link) => void links.sever(link)}
      onVote={(id, side, choice) => void links.vote(id, side, choice)}
    />
    <LinkRequestsSection
      requests={frame.pending}
      names={links.names}
      canPropose={frame.canPropose}
      pendingId={links.pendingId}
      onCompose={() => {
        links.composerMessage = undefined;
        composing = true;
      }}
      onVote={(id, side, choice) => void links.vote(id, side, choice)}
    />
    <LinksSection
      title="Historical links"
      description="Inactive or past links stay visible for lineage."
      items={frame.historical}
      emptyMessage="No historical links yet."
      allowSever={false}
      pendingId={null}
    />
    <LinkVoteHistory requests={frame.past} names={links.names} />
  {/if}
</section>

{#if composing}
  <LinkComposer
    pending={links.saving}
    message={links.composerMessage}
    search={links.search}
    onSubmit={(id, summary) => void submit(id, summary)}
    onClose={() => (composing = false)}
  />
{/if}

<script lang="ts">
  import SkeletonRows from '#lib/components/loading/SkeletonRows.svelte';
  import LinkComposer from '#lib/components/links/LinkComposer.svelte';
  import LinkRequestsSection from '#lib/components/links/LinkRequestsSection.svelte';
  import LinksSection from '#lib/components/links/LinksSection.svelte';
  import LinksPast from '#lib/components/links/LinksPast.svelte';
  import PlaceRow from '#lib/components/cards/PlaceRow.svelte';

  import type { DetailState } from './detailState.svelte.ts';
  import { LinksState } from './linksState.svelte.ts';

  import '#lib/components/styles/links.css';

  /**
   * web's DetailLinksTab for projects (Polish Pass mockups, 2026-11-16): Propose a link at the
   * top, the venue as a place row that opens the map, linked projects as rows, open votes as
   * cards, and past links and votes folded away.
   */
  interface Props {
    detail: DetailState;
    /** web's `?linkRequest=…`: the request (or sever vote) to open. */
    highlight?: string | null;
  }

  let { detail, highlight = null }: Props = $props();

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

{#snippet propose()}
  <button
    class="propose-link-button"
    type="button"
    onclick={() => {
      links.composerMessage = undefined;
      composing = true;
    }}
    ><svg aria-hidden="true" viewBox="0 0 24 24"><path d="M12 5v14M5 12h14" /></svg>Propose a link</button
  >
{/snippet}

<section class="links-tab">
  {#if venue && venue.toLowerCase() !== 'online'}
    <div class="links-venue"><PlaceRow place={venue} /></div>
  {/if}

  {#if links.error}<p class="tab-loading" role="alert">{links.error}</p>{/if}

  {#if !frame}
    <SkeletonRows count={2} kind="row" />
  {:else}
    <LinksSection
      action={frame.canPropose ? propose : undefined}
      items={frame.active}
      names={links.names}
      allowSever={frame.canPropose}
      pendingId={links.pendingId}
      {highlight}
      onSever={(link) => void links.sever(link)}
      onVote={(id, side, choice) => void links.vote(id, side, choice)}
    />
    <LinkRequestsSection
      requests={frame.pending}
      names={links.names}
      pendingId={links.pendingId}
      {highlight}
      onVote={(id, side, choice) => void links.vote(id, side, choice)}
    />
    <LinksPast links={frame.historical} votes={frame.past} names={links.names} />
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

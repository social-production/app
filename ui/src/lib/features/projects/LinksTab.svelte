<script lang="ts">
  import { t } from '#lib/i18n/index.ts';
  import SkeletonRows from '#lib/components/loading/SkeletonRows.svelte';
  import LinkComposer from '#lib/components/links/LinkComposer.svelte';
  import LinkRequestsSection from '#lib/components/links/LinkRequestsSection.svelte';
  import LinksSection from '#lib/components/links/LinksSection.svelte';
  import LinksPast from '#lib/components/links/LinksPast.svelte';

  import type { DetailState } from './detailState.svelte.ts';
  import QrCardSheet from './qrcard/QrCardSheet.svelte';
  import { LinksState } from './linksState.svelte.ts';

  import '#lib/components/styles/links.css';
  import '#lib/components/styles/ux/qrcard.css';

  /**
   * web's DetailLinksTab for projects (Polish Pass mockups, 2026-11-16): Propose a link at the
   * top (the place is on Details, owner 2026-11-18), linked projects as rows, open votes as
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
  // QR share cards (owner, 2026-11-24): a row at the top opens the download sheet.
  let qrOpen = $state(false);

  $effect(() => {
    void links.load();
  });

  const frame = $derived(links.frame);

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
    ><svg aria-hidden="true" viewBox="0 0 24 24"><path d="M12 5v14M5 12h14" /></svg>{t(
      'Propose a link',
    )}</button
  >
{/snippet}

<section class="links-tab">
  {#if detail.project?.charter}
    <button class="qr-card-row" type="button" onclick={() => (qrOpen = true)}>
      <span class="qr-card-icon" aria-hidden="true"
        ><svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8"
          ><rect x="4" y="4" width="6" height="6" rx="1" /><rect
            x="14"
            y="4"
            width="6"
            height="6"
            rx="1"
          /><rect x="4" y="14" width="6" height="6" rx="1" /><path
            d="M14 14h2v2h-2zM18 14h2M14 18h2M18 18h2v2"
            stroke-linecap="round"
          /></svg
        ></span
      >
      <span class="qr-card-copy">
        <strong>{t('Download QR card')}</strong>
        <small>{t('For posts, stories, stickers and flyers')}</small>
      </span>
      <svg aria-hidden="true" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"
        ><path d="m9 6 6 6-6 6" stroke-linecap="round" stroke-linejoin="round" /></svg
      >
    </button>
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

{#if qrOpen && detail.project?.charter}
  <QrCardSheet
    id={detail.project.id}
    charter={detail.project.charter}
    onClose={() => (qrOpen = false)}
  />
{/if}

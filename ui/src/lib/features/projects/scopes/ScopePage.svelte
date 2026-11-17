<script lang="ts">
  import LoadingLine from '#lib/components/loading/LoadingLine.svelte';
  import SkeletonRows from '#lib/components/loading/SkeletonRows.svelte';
  import SkeletonDetail from '#lib/components/loading/SkeletonDetail.svelte';
  import SpaceCard from '#lib/components/cards/SpaceCard.svelte';
  import InfiniteSentinel from '#lib/components/InfiniteSentinel.svelte';
  import PeopleSheet from '#lib/components/overlay/PeopleSheet.svelte';
  import PlatformBoardPanel from '#lib/components/board/PlatformBoardPanel.svelte';
  import EmptyState from '#lib/components/empty/EmptyState.svelte';
  import FeedEnd from '#lib/components/empty/FeedEnd.svelte';
  import { keepCard, type FeedType } from '#lib/domain/feedTypes.ts';
  import { keepFeedFilters, readFeedFilters } from '#lib/stores/feedFilters.ts';
  import { parseInvite, scopeTag, type ScopeKind } from '#lib/domain/scopes.ts';

  import FeedFilterBar from '../FeedFilterBar.svelte';

  import { BoardState } from './boardState.svelte.ts';
  import { ScopeArt } from './scopeArt.svelte.ts';

  import CommunityInvitePanel from './CommunityInvitePanel.svelte';
  import ScopeHeader from './ScopeHeader.svelte';
  import ScopeSettingsSheet from './ScopeSettingsSheet.svelte';
  import { ScopeState, redeemInvite, sendInvite } from './scopeState.svelte.ts';
  import ScopeVoteSheet from './ScopeVoteSheet.svelte';

  import { goto } from '$app/navigation';
  import { resolve } from '$app/paths';

  import '#lib/components/styles/scopes.css';

  /**
   * web's DirectoryFeedLayout + ScopeDirectoryHeader for a channel or community (ADR 0068):
   * name, join/members, settings, description, then the items tagged with it; a Vote button
   * while a decision waits for you. A private community (ADR 0068 part d) shows web's lock
   * and invite panel; an invite link opens it by the seed in its fragment. Its feed is the
   * events listed on it, for members (ADR 0069).
   */
  interface Props {
    kind: ScopeKind;
    slug: string;
    hash?: string;
    /** The Platform channel's `?board=1`: open the moderator board (ADR 0095). */
    board?: boolean;
  }
  let { kind, slug, hash = '', board = false }: Props = $props();

  // The route re-creates this page per slug and fragment ({#key}), so reading the props once
  // is intended. A private community's invite link carries its seed in the fragment (ADR 0068).
  // svelte-ignore state_referenced_locally
  const s = new ScopeState(kind, slug, parseInvite(hash));
  let membersOpen = $state(false);
  let inviteOpen = $state(!!s.invite);
  let settingsOpen = $state(false);
  let votesOpen = $state(false);
  let boardOpen = $state(false);
  $effect(() => {
    if (board) boardOpen = true;
  });
  const boardState = new BoardState();
  // The picture and cover the charter names now (ADR 0109).
  const art = new ScopeArt();
  $effect(() => {
    void art.show(s.project?.charter?.picture, s.project?.charter?.cover);
  });
  const waiting = $derived(s.gov?.votable.length ?? 0);
  const noun = $derived(kind === 'channel' ? 'channel' : 'community');
  // The feed bar's Type (Scope Filters mockups §1), applied to the loaded cards.
  // The filters last across reloads, on this device (one set for every channel and community).
  const saved = readFeedFilters('scope', { type: 'all', sort: 'trending', window: 'all' });
  s.feed.sort = saved.sort;
  s.feed.window = saved.window;
  let type = $state<FeedType>(saved.type);
  $effect(() => keepFeedFilters('scope', { type, sort: s.feed.sort, window: s.feed.window }));
  const items = $derived(s.feed.items.filter((card) => keepCard(card, type)));
  const narrowed = $derived(type !== 'all' || s.feed.window !== 'all');
  const title = $derived(s.platform ? 'Platform' : (s.scope?.handle ?? slug));
  const threadHref = $derived(
    `${resolve('/create/thread')}?tag=${encodeURIComponent(scopeTag(kind, slug))}`,
  );
  function clearFilters(): void {
    type = 'all';
    s.feed.window = 'all';
    void s.feed.reload();
  }

  $effect(() => {
    void s.open();
  });

  $effect(() => {
    // The header shows the moderators' faces and the weekly count, so the board loads at once.
    if (s.platform && !boardState.board) void boardState.load();
  });

  async function redeem(): Promise<void> {
    const joined = await redeemInvite(s);
    if (!joined) return;
    inviteOpen = false;
    if (joined.id === s.scope?.id) return;
    const page = resolve('/communities/[slug]', { slug: joined.slug });
    // eslint-disable-next-line svelte/no-navigation-without-resolve -- resolved above
    await goto(`${page}#invite=${joined.id}`);
  }
</script>

{#snippet invitePanel()}
  <CommunityInvitePanel
    {s}
    onRedeem={() => void redeem()}
    onInvite={(c) => void sendInvite(s, c)}
  />
{/snippet}

<section class="directory-page">
  {#if !s.loaded}
    <LoadingLine /><SkeletonDetail />
  {:else if !s.scope}
    <section class="info-card">
      <p>{s.error ?? `No ${noun} has the handle "${slug}" yet.`}</p>
    </section>
  {:else}
    {@const scope = s.scope}
    <ScopeHeader
      {s}
      {art}
      {kind}
      {waiting}
      {invitePanel}
      bind:membersOpen
      bind:inviteOpen
      bind:settingsOpen
      bind:boardOpen
      board={boardState.board}
      people={boardState.people}
      onVotes={() => (votesOpen = true)}
    />

    {#if !s.scope?.isPrivate}<FeedFilterBar feed={s.feed} name={title} bind:type />{/if}

    <div class="stack">
      {#if s.feed.loading && items.length === 0}
        <SkeletonRows count={3} />
      {:else if items.length === 0 && narrowed}
        <EmptyState
          icon="filter"
          title="Nothing matches these filters"
          text="Try a longer time or another type."
        >
          <button class="empty-action" type="button" onclick={clearFilters}>Clear filters</button>
        </EmptyState>
      {:else if items.length === 0 && s.platform}
        <EmptyState
          icon="globe"
          title="Nothing for the whole network yet"
          text="Projects and events tagged #platform show here."
        >
          <!-- eslint-disable-next-line svelte/no-navigation-without-resolve -- resolved, plus a query -->
          <a class="empty-action primary" href={`${resolve('/create/project')}?tag=platform`}
            >Start a platform project</a
          >
        </EmptyState>
      {:else if items.length === 0}
        <EmptyState
          icon={kind === 'channel' ? 'hash' : 'people'}
          title={`Nothing in ${title} yet`}
          text={`Posts, events and projects tagged with this ${noun} show up here.`}
        >
          <!-- eslint-disable-next-line svelte/no-navigation-without-resolve -- resolved, plus a query -->
          <a class="empty-action primary" href={threadHref}>Start a thread</a>
        </EmptyState>
      {:else}
        {#each items as card (card.id)}
          <SpaceCard {card} onSignal={(next) => s.feed.signal(card, next)} />
        {/each}
        <InfiniteSentinel
          disabled={!s.feed.hasMore || s.feed.loading}
          onMore={() => void s.feed.more()}
        />
        {#if !s.feed.hasMore}<FeedEnd />{/if}
      {/if}
    </div>

    <PeopleSheet
      open={membersOpen}
      title="Members"
      emptyCopy="No members yet."
      people={s.members}
      onClose={() => (membersOpen = false)}
    />
    {#if s.platform}
      <PlatformBoardPanel
        open={boardOpen}
        board={boardState.board}
        people={boardState.people}
        me={boardState.me}
        canVolunteer={boardState.canVolunteer}
        onVolunteer={boardState.volunteer}
        onVote={boardState.vote}
        onClose={() => (boardOpen = false)}
      />
    {/if}
    {#if s.gov}
      <ScopeSettingsSheet
        open={settingsOpen}
        {kind}
        handle={scope.handle}
        description={s.project?.charter?.description ?? scope.description}
        isMember={!!s.project?.amMember}
        gov={s.gov}
        memberName={s.memberName}
        {art}
        isPrivate={!!scope.isPrivate}
        organized={s.project?.governance === 'organizer_controlled'}
        amOrganizer={!!s.project?.amOrganizer}
        organizerNames={(s.project?.organizers ?? []).map(s.memberName)}
        members={s.others}
        onChanged={() => void s.reload()}
        onClose={() => (settingsOpen = false)}
      />
      <ScopeVoteSheet
        {art}
        open={votesOpen}
        gov={s.gov}
        memberName={s.memberName}
        onClose={() => (votesOpen = false)}
      />
    {/if}
  {/if}
</section>

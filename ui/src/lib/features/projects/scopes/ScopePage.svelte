<script lang="ts">
  import SpaceCard from '#lib/components/cards/SpaceCard.svelte';
  import InfiniteSentinel from '#lib/components/InfiniteSentinel.svelte';
  import PeopleSheet from '#lib/components/overlay/PeopleSheet.svelte';
  import PlatformBoardPanel from '#lib/components/board/PlatformBoardPanel.svelte';
  import { parseInvite, type ScopeKind } from '#lib/domain/scopes.ts';

  import { BoardState } from './boardState.svelte.ts';

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
  const waiting = $derived(s.gov?.votable.length ?? 0);
  const noun = $derived(kind === 'channel' ? 'channel' : 'community');

  $effect(() => {
    void s.open();
  });

  $effect(() => {
    if (s.platform && boardOpen && !boardState.board) void boardState.load();
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
    <section class="info-card"><p>Loading {noun}…</p></section>
  {:else if !s.scope}
    <section class="info-card">
      <p>{s.error ?? `No ${noun} has the handle "${slug}" yet.`}</p>
    </section>
  {:else}
    {@const scope = s.scope}
    <ScopeHeader
      {s}
      {kind}
      {waiting}
      {invitePanel}
      bind:membersOpen
      bind:inviteOpen
      bind:settingsOpen
      bind:boardOpen
      onVotes={() => (votesOpen = true)}
    />

    <div class="stack">
      {#if s.feed.loading && s.feed.items.length === 0}
        <section class="info-card"><p>Loading feed…</p></section>
      {:else if s.feed.items.length === 0}
        <section class="info-card">
          {#if s.platform}
            <p>No platform activity yet.</p>
          {:else}
            <p>
              No tagged posts, help, or events in this {noun} yet. Tag content with this {noun} to show
              it here.
            </p>
          {/if}
        </section>
      {:else}
        {#each s.feed.items as card (card.id)}
          <SpaceCard {card} onSignal={(next) => s.feed.signal(card, next)} />
        {/each}
        <InfiniteSentinel
          disabled={!s.feed.hasMore || s.feed.loading}
          onMore={() => void s.feed.more()}
        />
        {#if !s.feed.hasMore}<p class="end-copy">You're caught up.</p>{/if}
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
        isPrivate={!!scope.isPrivate}
        organized={s.project?.governance === 'organizer_controlled'}
        amOrganizer={!!s.project?.amOrganizer}
        organizerNames={(s.project?.organizers ?? []).map(s.memberName)}
        members={s.others}
        onChanged={() => void s.reload()}
        onClose={() => (settingsOpen = false)}
      />
      <ScopeVoteSheet
        open={votesOpen}
        gov={s.gov}
        memberName={s.memberName}
        onClose={() => (votesOpen = false)}
      />
    {/if}
  {/if}
</section>

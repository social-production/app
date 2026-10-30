<script lang="ts">
  import SpaceCard from '#lib/components/cards/SpaceCard.svelte';
  import IconPopoverMenu from '#lib/components/IconPopoverMenu.svelte';
  import SurfaceIcon from '#lib/components/icons/SurfaceIcon.svelte';
  import ToolbarIcon from '#lib/components/icons/ToolbarIcon.svelte';
  import InfiniteSentinel from '#lib/components/InfiniteSentinel.svelte';
  import MembershipSplitButton from '#lib/components/MembershipSplitButton.svelte';
  import PeopleSheet from '#lib/components/overlay/PeopleSheet.svelte';
  import { parseInvite, type ScopeKind } from '#lib/domain/scopes.ts';

  import CommunityInvitePanel from './CommunityInvitePanel.svelte';
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
   * and invite panel; an invite link opens it by the seed in its fragment.
   */
  let { kind, slug, hash = '' }: { kind: ScopeKind; slug: string; hash?: string } = $props();

  // The route re-creates this page per slug and fragment ({#key}), so reading the props once
  // is intended. A private community's invite link carries its seed in the fragment (ADR 0068).
  // svelte-ignore state_referenced_locally
  const s = new ScopeState(kind, slug, parseInvite(hash));
  let membersOpen = $state(false);
  let inviteOpen = $state(!!s.invite);
  let settingsOpen = $state(false);
  let votesOpen = $state(false);
  const waiting = $derived(s.gov?.votable.length ?? 0);
  const noun = $derived(kind === 'channel' ? 'channel' : 'community');

  $effect(() => {
    void s.open();
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
    <section class="scope-header">
      <div class="title-row">
        <div class="title-block">
          <span class="scope-avatar" style="--avatar-size: 40px" aria-hidden="true">
            <SurfaceIcon icon={kind} size="md" />
          </span>
          <h1>{scope.handle}</h1>
          {#if scope.isPrivate}
            <span class="private-mark" title="Private: members only" aria-label="Private">
              <svg aria-hidden="true" viewBox="0 0 24 24" width="14" height="14">
                <path
                  d="M7 11V8a5 5 0 0 1 10 0v3M6 11h12v9H6z"
                  fill="none"
                  stroke="currentColor"
                  stroke-width="2"
                  stroke-linejoin="round"
                />
              </svg>
            </span>
          {/if}
        </div>
        <div class="action-row">
          {#if scope.isPrivate && !s.project?.amMember}
            <IconPopoverMenu
              open={inviteOpen}
              active={inviteOpen}
              chip
              ariaLabel={`Join ${scope.handle} with invite, ${s.project?.members.length ?? scope.members} members`}
              menuLabel="Join with invite"
              minWidth={300}
              onOpenChange={(o) => (inviteOpen = o)}
            >
              {#snippet icon()}
                <ToolbarIcon name="person-plus" />
                <span>{s.project?.members.length ?? scope.members}</span>
              {/snippet}
              {@render invitePanel()}
            </IconPopoverMenu>
          {:else}
            <MembershipSplitButton
              joined={!!s.project?.amMember}
              count={s.project?.members.length ?? scope.members}
              pending={s.joining || !s.project}
              {membersOpen}
              noun={scope.handle}
              onToggleJoin={() => void s.toggleJoin()}
              onOpenMembers={() => (membersOpen = true)}
            />
          {/if}
          {#if scope.isPrivate && s.project?.amMember}
            <IconPopoverMenu
              open={inviteOpen}
              active={inviteOpen}
              ariaLabel={`Invite people to ${scope.handle}`}
              menuLabel="Invite people"
              minWidth={300}
              onOpenChange={(o) => (inviteOpen = o)}
            >
              {#snippet icon()}<ToolbarIcon name="person-plus" />{/snippet}
              {@render invitePanel()}
            </IconPopoverMenu>
          {/if}
          <button
            aria-label="Settings"
            title="Settings"
            class="icon-action"
            class:menu-active={settingsOpen}
            type="button"
            onclick={() => (settingsOpen = true)}
          >
            <svg aria-hidden="true" viewBox="0 0 24 24" width="18" height="18">
              <path
                d="M10.3 2h3.4l.5 2.4c.5.2 1 .4 1.5.6l2.1-1.2 2.4 2.4-1.2 2.1c.2.5.4 1 .6 1.5L22 10.3v3.4l-2.4.5c-.2.5-.4 1-.6 1.5l1.2 2.1-2.4 2.4-2.1-1.2c-.5.2-1 .4-1.5.6L13.7 22h-3.4l-.5-2.4c-.5-.2-1-.4-1.5-.6l-2.1 1.2-2.4-2.4 1.2-2.1c-.2-.5-.4-1-.6-1.5L2 13.7v-3.4l2.4-.5c.2-.5.4-1 .6-1.5L3.8 6.2l2.4-2.4 2.1 1.2c.5-.2 1-.4 1.5-.6L10.3 2Zm1.7 6.2A3.8 3.8 0 1 0 12 15.8 3.8 3.8 0 0 0 12 8.2Z"
                fill="currentColor"
              ></path>
            </svg>
          </button>
        </div>
      </div>
      <div class="scope-intro-prose">
        <p>{s.project?.charter?.description ?? scope.description}</p>
      </div>
      {#if waiting > 0}
        <!-- Sits just left of the + button while a decision is waiting for this viewer. -->
        <button
          class="vote-fab"
          type="button"
          aria-label={`Vote: ${waiting} waiting for your vote`}
          onclick={() => (votesOpen = true)}
        >
          Vote
          <span class="vote-fab-count">{waiting}</span>
        </button>
      {/if}
    </section>

    <div class="stack">
      {#if !scope.isPrivate && s.feed.loading && s.feed.items.length === 0}
        <section class="info-card"><p>Loading feed…</p></section>
      {:else if s.feed.items.length === 0}
        <section class="info-card">
          <p>
            No tagged posts, help, or events in this {noun} yet. Tag content with this {noun} to show
            it here.
          </p>
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

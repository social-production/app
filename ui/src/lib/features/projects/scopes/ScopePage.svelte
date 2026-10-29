<script lang="ts">
  import SpaceCard from '#lib/components/cards/SpaceCard.svelte';
  import SurfaceIcon from '#lib/components/icons/SurfaceIcon.svelte';
  import InfiniteSentinel from '#lib/components/InfiniteSentinel.svelte';
  import MembershipSplitButton from '#lib/components/MembershipSplitButton.svelte';
  import PeopleSheet from '#lib/components/overlay/PeopleSheet.svelte';
  import type { ScopeKind } from '#lib/domain/scopes.ts';

  import { ScopeState } from './scopeState.svelte.ts';

  import '#lib/components/styles/scopes.css';

  /**
   * web's DirectoryFeedLayout + ScopeDirectoryHeader for a channel or community (ADR 0068):
   * name, join/members, description, then the items tagged with it. Settings and votes come
   * in M5d part (e); private communities' invites in part (d).
   */
  let { kind, slug }: { kind: ScopeKind; slug: string } = $props();

  // The route re-creates this page per slug ({#key}), so reading the props once is intended.
  // svelte-ignore state_referenced_locally
  const s = new ScopeState(kind, slug);
  let membersOpen = $state(false);
  const noun = $derived(kind === 'channel' ? 'channel' : 'community');

  $effect(() => {
    void s.open();
  });
</script>

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
        </div>
        <div class="action-row">
          <MembershipSplitButton
            joined={!!s.project?.amMember}
            count={s.project?.members.length ?? scope.members}
            pending={s.joining || !s.project}
            {membersOpen}
            noun={scope.handle}
            onToggleJoin={() => void s.toggleJoin()}
            onOpenMembers={() => (membersOpen = true)}
          />
        </div>
      </div>
      <div class="scope-intro-prose"><p>{scope.description}</p></div>
    </section>

    <div class="stack">
      {#if s.feed.loading && s.feed.items.length === 0}
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
  {/if}
</section>

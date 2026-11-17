<script lang="ts">
  import { untrack } from 'svelte';

  import SkeletonRows from '#lib/components/loading/SkeletonRows.svelte';
  import CommentActivityCard from '#lib/components/cards/CommentActivityCard.svelte';
  import PersonalSpaceCard from '#lib/components/cards/PersonalSpaceCard.svelte';
  import EmptyState from '#lib/components/empty/EmptyState.svelte';
  import FeedEnd from '#lib/components/empty/FeedEnd.svelte';
  import ToolbarIcon from '#lib/components/icons/ToolbarIcon.svelte';
  import { FEED_WINDOWS } from '#lib/domain/feedTypes.ts';
  import InfiniteSentinel from '#lib/components/InfiniteSentinel.svelte';
  import OverlaySheet from '#lib/components/overlay/OverlaySheet.svelte';
  import SegmentedChoice from '#lib/components/SegmentedChoice.svelte';
  import type { CommentActivity } from '#lib/domain/commentNotes.ts';
  import {
    forAudience,
    profileRows,
    type ProfileAudience,
    type ProfileTab,
  } from '#lib/domain/profileFeed.ts';
  import type { FeedSort } from '#lib/domain/projects.ts';
  import { cachedPerson } from '#lib/services/people.ts';
  import { commentsOf } from '#lib/services/profileNotes.ts';

  import { connection } from '#lib/stores/connection.svelte.ts';

  import { FeedState } from './feedState.svelte.ts';

  /**
   * A person's profile feed (Profiles and Comments mockups §2): Global's tab row (All, Posts,
   * Comments, Projects) with one filter button, the Global rows, and their comments as
   * comment rows (ADR 0107).
   */
  let { agent }: { agent?: string } = $props();

  // Your own profile (/profile) passes no key: it is the signed-in person's.
  const me = connection.current.status === 'connected' ? connection.current.agent : '';
  // The page is re-created per person ({#key}), so reading `agent` once is intended.
  // svelte-ignore state_referenced_locally
  const who = agent ?? me;
  const feed = new FeedState(() => '', who);
  let comments = $state<CommentActivity[]>([]);
  let tab = $state<ProfileTab>('all');
  let open = $state(false);
  let audience = $state<ProfileAudience>('all');
  let name = $state('');
  const audiences: { value: ProfileAudience; label: string }[] = [
    { value: 'all', label: 'Everything' },
    { value: 'public', label: 'Public only' },
  ];
  // svelte-ignore state_referenced_locally
  const own = agent === undefined; // re-created per person, as `who` above
  const sheetTitle = $derived(name ? `Show on ${name}'s profile` : 'Show on this profile');

  const tabs: { value: ProfileTab; label: string }[] = [
    { value: 'all', label: 'All' },
    { value: 'posts', label: 'Posts' },
    { value: 'comments', label: 'Comments' },
    { value: 'projects', label: 'Projects' },
  ];
  const sorts: { value: FeedSort; label: string }[] = [
    { value: 'recent', label: 'Newest' },
    { value: 'top', label: 'Top' },
    { value: 'oldest', label: 'Oldest' },
  ];
  // Each tab's empty line (Scope Filters mockups §2), naming the person.
  const empty = $derived.by(() => {
    const who = own ? 'you' : name || 'they';
    const kinds = {
      all: 'posts or projects',
      posts: 'posts',
      comments: 'comments',
      projects: 'projects',
    };
    const noun = kinds[tab];
    return {
      title: `No ${noun} yet`,
      text: own
        ? `What you share shows up here.`
        : `When ${who} ${tab === 'comments' ? 'comments' : 'posts or starts something'}, it shows here.`,
    };
  });
  const rows = $derived(
    forAudience(profileRows(feed.items, comments, tab, feed.sort), own ? audience : 'all'),
  );

  $effect(() => {
    untrack(() => {
      feed.sort = 'recent';
      void feed.reload();
      void cachedPerson(who).then(
        (p) => (name = p.profile?.name ?? ''),
        () => undefined,
      );
      void commentsOf([who]).then(
        (found) => (comments = found),
        () => undefined,
      );
    });
  });
</script>

<header class="feed-header profile-feed-header">
  <div class="feed-tabs">
    <div class="feed-tablist" role="tablist" aria-label="Show on this profile">
      {#each tabs as t (t.value)}
        <button
          type="button"
          role="tab"
          aria-selected={tab === t.value}
          class:on={tab === t.value}
          onclick={() => (tab = t.value)}>{t.label}</button
        >
      {/each}
    </div>
    <button
      type="button"
      class="feed-tune"
      class:narrowed={feed.sort !== 'recent' || feed.window !== 'all'}
      aria-label="Profile filters"
      aria-haspopup="dialog"
      title="Profile filters"
      onclick={() => (open = true)}
    >
      <ToolbarIcon name="filter" />
    </button>
  </div>
</header>

<section class="feed-stack">
  {#if feed.error}
    <EmptyState icon="info" title="This profile didn't load" text={feed.error} />
  {:else if feed.loading && rows.length === 0}
    <SkeletonRows count={2} />
  {:else if rows.length === 0 && feed.window !== 'all'}
    <EmptyState icon="filter" title="Nothing in this time" text="Try a longer time.">
      <button
        class="empty-action"
        type="button"
        onclick={() => ((feed.window = 'all'), void feed.reload())}>Show all time</button
      >
    </EmptyState>
  {:else if rows.length === 0}
    <EmptyState
      icon={tab === 'comments' ? 'message' : tab === 'projects' ? 'layers' : 'doc'}
      title={empty.title}
      text={empty.text}
    />
  {:else}
    {#each rows as row (row.key)}
      {#if 'comment' in row}
        <CommentActivityCard comment={row.comment} you={row.comment.author === me} />
      {:else}
        <PersonalSpaceCard card={row.card} onSignal={(next) => feed.signal(row.card, next)} />
      {/if}
    {/each}
    {#if tab !== 'comments'}
      <InfiniteSentinel disabled={!feed.hasMore || feed.loading} onMore={() => void feed.more()} />
    {/if}
    {#if !feed.hasMore || tab === 'comments'}<FeedEnd />{/if}
  {/if}
</section>

<OverlaySheet
  {open}
  title={sheetTitle}
  labelledById="profile-filter-title"
  rootClass="shell feed-filter-sheet"
  onClose={() => (open = false)}
>
  <div class="feed-filter-groups">
    <section>
      <h3>Sort</h3>
      <SegmentedChoice
        value={feed.sort}
        ariaLabel="Sort profile feed"
        options={sorts}
        onChange={(v) => ((feed.sort = v), void feed.reload())}
      />
    </section>
    <section>
      <h3>When</h3>
      <SegmentedChoice
        value={feed.window}
        ariaLabel="Profile time window"
        options={FEED_WINDOWS}
        onChange={(v) => ((feed.window = v), void feed.reload())}
      />
    </section>
    {#if own}
      <section>
        <h3>Who can see</h3>
        <SegmentedChoice
          value={audience}
          ariaLabel="Who can see"
          options={audiences}
          onChange={(v) => (audience = v)}
        />
      </section>
    {/if}
  </div>
  {#snippet footer()}
    <div class="sheet-actions">
      <button class="sheet-submit" type="button" onclick={() => (open = false)}>Show results</button
      >
    </div>
  {/snippet}
</OverlaySheet>

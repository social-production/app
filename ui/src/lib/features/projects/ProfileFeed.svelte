<script lang="ts">
  import { untrack } from 'svelte';

  import CommentActivityCard from '#lib/components/cards/CommentActivityCard.svelte';
  import PersonalSpaceCard from '#lib/components/cards/PersonalSpaceCard.svelte';
  import ToolbarIcon from '#lib/components/icons/ToolbarIcon.svelte';
  import InfiniteSentinel from '#lib/components/InfiniteSentinel.svelte';
  import OverlaySheet from '#lib/components/overlay/OverlaySheet.svelte';
  import SegmentedChoice from '#lib/components/SegmentedChoice.svelte';
  import type { CommentActivity } from '#lib/domain/commentNotes.ts';
  import { profileRows, type ProfileTab } from '#lib/domain/profileFeed.ts';
  import type { FeedSort } from '#lib/domain/projects.ts';
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
  const rows = $derived(profileRows(feed.items, comments, tab, feed.sort));

  $effect(() => {
    untrack(() => {
      feed.sort = 'recent';
      void feed.reload();
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
      class:narrowed={feed.sort !== 'recent'}
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
    <section class="empty-card"><p>{feed.error}</p></section>
  {:else if feed.loading && rows.length === 0}
    <section class="empty-card"><p>Loading activity…</p></section>
  {:else if rows.length === 0}
    <section class="empty-card">
      <p>{tab === 'comments' ? 'No comments yet.' : 'Nothing here yet.'}</p>
    </section>
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
    {#if !feed.hasMore || tab === 'comments'}<p class="end-copy">You're caught up.</p>{/if}
  {/if}
</section>

<OverlaySheet
  {open}
  title="Show on this profile"
  labelledById="profile-filter-title"
  rootClass="shell feed-filter-sheet"
  onClose={() => (open = false)}
>
  <div class="feed-filter-groups">
    <section>
      <h3>Order</h3>
      <SegmentedChoice
        value={feed.sort}
        ariaLabel="Sort profile feed"
        options={sorts}
        onChange={(v) => ((feed.sort = v), void feed.reload())}
      />
    </section>
  </div>
  {#snippet footer()}
    <div class="sheet-actions">
      <button class="sheet-submit" type="button" onclick={() => (open = false)}>Show results</button
      >
    </div>
  {/snippet}
</OverlaySheet>

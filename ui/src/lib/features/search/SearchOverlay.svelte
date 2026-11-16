<script lang="ts">
  import { untrack } from 'svelte';

  import AvatarBadge from '#lib/components/icons/AvatarBadge.svelte';
  import ToolbarIcon from '#lib/components/icons/ToolbarIcon.svelte';
  import type { Scope } from '#lib/domain/scopes.ts';
  import type { SearchResultItem, SearchResultKind } from '#lib/domain/search.ts';
  import { listScopes } from '#lib/services/scopes.ts';
  import { searchOverlay } from '#lib/stores/searchOverlay.svelte.ts';

  import { goto } from '$app/navigation';
  import { resolve } from '$app/paths';

  import { createLiveSearchScheduler } from './liveSearch.ts';
  import { clearSearches, forgetSearch, readRecent, rememberSearch } from './recentSearches.ts';

  /**
   * The full-screen search (Chats, Search and Reports mockups §3, like Reddit's and
   * Substack's): before you type, your recent searches (this browser only) and channels to
   * explore; as you type, suggestions grouped by kind; Enter or "Search for …" opens results.
   */
  let q = $state('');
  let field = $state<HTMLInputElement>();
  let recent = $state<string[]>([]);
  let explore = $state<Scope[]>([]);
  let found = $state<SearchResultItem[]>([]);
  let loading = $state(false);
  const live = createLiveSearchScheduler();

  $effect(() => {
    if (!searchOverlay.open) return;
    untrack(() => {
      q = searchOverlay.query;
      recent = readRecent();
      if (q.trim()) schedule();
      if (explore.length === 0)
        void listScopes('', 50).then(
          (all) => (explore = all.sort((a, b) => b.members - a.members).slice(0, 5)),
          () => undefined,
        );
    });
  });

  const GROUPS: { label: string; kinds: SearchResultKind[] }[] = [
    { label: 'Channels and communities', kinds: ['channel', 'community'] },
    { label: 'People', kinds: ['profile'] },
    { label: 'Posts and projects', kinds: ['project', 'thread', 'event', 'help-request', 'post'] },
  ];
  const groups = $derived(
    GROUPS.map((g) => ({
      ...g,
      items: found.filter((r) => g.kinds.includes(r.kind)).slice(0, 4),
    })).filter((g) => g.items.length > 0),
  );
  const ICONS: Record<SearchResultKind, string> = {
    channel: 'message',
    community: 'people',
    project: 'lightbulb',
    thread: 'message',
    event: 'calendar',
    'help-request': 'person-plus',
    post: 'pencil',
    profile: 'user',
  };
  const LABELS: Record<SearchResultKind, string> = {
    channel: 'Channel',
    community: 'Community',
    project: 'Project',
    thread: 'Thread',
    event: 'Event',
    'help-request': 'Help request',
    post: 'Post',
    profile: 'Person',
  };

  function schedule(): void {
    live.schedule(q, 'all', (results, busy) => ((found = results), (loading = busy)));
  }

  /** The title split around the first match, so the match can be bold. */
  function parts(title: string): [string, string, string] {
    const at = title.toLowerCase().indexOf(q.trim().toLowerCase());
    if (at < 0 || !q.trim()) return [title, '', ''];
    const end = at + q.trim().length;
    return [title.slice(0, at), title.slice(at, end), title.slice(end)];
  }

  function close(): void {
    live.cancel();
    searchOverlay.open = false;
  }

  function go(href: string, remember: string): void {
    rememberSearch(remember);
    close();
    // eslint-disable-next-line svelte/no-navigation-without-resolve -- the app's own paths
    void goto(href);
  }

  function searchFor(text: string): void {
    if (text.trim()) go(`${resolve('/search')}?q=${encodeURIComponent(text.trim())}`, text);
  }

  function focusOnOpen(node: HTMLInputElement): void {
    node.focus();
  }
</script>

<svelte:window onkeydown={(event) => searchOverlay.open && event.key === 'Escape' && close()} />

{#if searchOverlay.open}
  <div class="search-overlay" role="dialog" aria-modal="true" aria-label="Search">
    <div class="so-panel">
      <form
        class="so-bar"
        role="search"
        onsubmit={(event) => {
          event.preventDefault();
          searchFor(q);
        }}
      >
        <label class="so-field">
          <ToolbarIcon name="search" />
          <input
            bind:this={field}
            use:focusOnOpen
            bind:value={q}
            oninput={schedule}
            type="search"
            enterkeyhint="search"
            aria-label="Search"
            placeholder="Search Social Production"
          />
          {#if q}
            <button
              class="so-clear"
              type="button"
              aria-label="Clear"
              onclick={() => {
                q = '';
                schedule();
                field?.focus();
              }}><ToolbarIcon name="x" /></button
            >
          {/if}
        </label>
        <button class="so-cancel" type="button" onclick={close}>Cancel</button>
      </form>

      <div class="so-body">
        {#if !q.trim()}
          {#if recent.length > 0}
            <div class="so-head">
              <b>Recent</b><button type="button" onclick={() => (recent = clearSearches())}
                >Clear all</button
              >
            </div>
            {#each recent as r (r)}
              <div class="so-row">
                <button class="so-main" type="button" onclick={() => searchFor(r)}
                  ><span class="so-icon"><ToolbarIcon name="clock" /></span><span class="so-text"
                    >{r}</span
                  ></button
                >
                <button
                  class="so-x"
                  type="button"
                  aria-label={`Remove ${r}`}
                  onclick={() => (recent = forgetSearch(r))}><ToolbarIcon name="x" /></button
                >
              </div>
            {/each}
          {/if}
          {#if explore.length > 0}
            <div class="so-head"><b>Channels to explore</b></div>
            {#each explore as s (s.id)}
              <button
                class="so-main"
                type="button"
                onclick={() =>
                  go(
                    s.kind === 'channel'
                      ? resolve('/channels/[slug]', { slug: s.slug })
                      : resolve('/communities/[slug]', { slug: s.slug }),
                    s.handle,
                  )}
              >
                <span class="so-glyph"
                  ><ToolbarIcon name={s.kind === 'channel' ? 'message' : 'people'} /></span
                >
                <span class="so-text"
                  >{s.handle}<small
                    >{s.kind === 'channel' ? 'Channel' : 'Community'} · {s.members}
                    {s.members === 1 ? 'member' : 'members'}</small
                  ></span
                >
              </button>
            {/each}
          {/if}
        {:else}
          <button class="so-main" type="button" onclick={() => searchFor(q)}>
            <span class="so-icon"><ToolbarIcon name="search" /></span>
            <span class="so-text"><span>Search for <b>“{q.trim()}”</b></span></span>
          </button>
          {#each groups as g (g.label)}
            <div class="so-head"><b>{g.label}</b></div>
            {#each g.items as r (r.id)}
              {@const [before, match, after] = parts(r.title)}
              <button class="so-main" type="button" onclick={() => go(r.href, q)}>
                {#if r.kind === 'profile'}<AvatarBadge size="md" name={r.title} />{:else}<span
                    class="so-glyph"><ToolbarIcon name={ICONS[r.kind]} /></span
                  >{/if}
                <span class="so-text"
                  ><span>{before}<b>{match}</b>{after}</span><small>{LABELS[r.kind]}</small></span
                >
              </button>
            {/each}
          {/each}
          {#if groups.length === 0}
            <p class="so-quiet">
              {loading ? 'Searching…' : 'No quick matches. Press Search to look everywhere.'}
            </p>
          {/if}
        {/if}
      </div>
    </div>
  </div>
{/if}

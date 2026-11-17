<script lang="ts">
  import { untrack } from 'svelte';

  import SurfaceIcon from '#lib/components/icons/SurfaceIcon.svelte';
  import OverlaySheet from '#lib/components/overlay/OverlaySheet.svelte';
  import { explain } from '#lib/services/errors.ts';
  import { joinProject } from '#lib/services/projects.ts';
  import { listScopes, myScopes as loadMyScopes } from '#lib/services/scopes.ts';
  import { requireViewer } from '#lib/stores/authNotice.svelte.ts';
  import { myScopes, showMyScopes } from '#lib/stores/myScopes.svelte.ts';

  import { resolve } from '$app/paths';
  import type { Scope, ScopeKind } from '#lib/domain/scopes.ts';

  import { scopeHref, withPictures } from './myScopesLoop.ts';

  import '#lib/components/styles/scopes.css';

  /** web's ScopeDiscoverSheet (left rail "Discover"), as drawn (Plan, Phase and Feedback
   * Screens, 6): a pill search field, a segmented filter, rows with the kind, members and a
   * Join pill, and an offer to start one. The private communities you're in are added from
   * your own (nothing lists them). */
  interface Props {
    open: boolean;
    /** Words typed into the rail's field. */
    initialQuery?: string;
    onClose: () => void;
    /** Called after a row is chosen (closes the phone rails). */
    onNavigate: () => void;
  }

  let { open, initialQuery = '', onClose, onNavigate }: Props = $props();

  type DiscoverFilter = 'all' | ScopeKind;

  const filters: { value: DiscoverFilter; label: string }[] = [
    { value: 'all', label: 'All' },
    { value: 'channel', label: 'Channels' },
    { value: 'community', label: 'Communities' },
  ];

  let query = $state('');
  let activeFilter = $state<DiscoverFilter>('all');
  let scopes = $state<Scope[]>([]);
  let loading = $state(false);
  let loadError = $state('');
  let requestId = 0;

  // Opening starts from the rail's words, if any.
  $effect(() => {
    if (open) query = untrack(() => initialQuery);
  });

  $effect(() => {
    if (!open) {
      query = '';
      activeFilter = 'all';
      return;
    }
    const needle = query.trim();
    const timer = setTimeout(() => void load(needle), needle ? 180 : 0);
    return () => clearTimeout(timer);
  });

  async function load(needle: string): Promise<void> {
    const current = ++requestId;
    if (scopes.length === 0) loading = true;
    loadError = '';
    try {
      const found = await withPictures(await listScopes(needle, 200));
      if (current === requestId) scopes = found;
    } catch {
      if (current === requestId) loadError = 'Could not load channels and communities.';
    } finally {
      if (current === requestId) loading = false;
    }
  }

  const joined = $derived(
    new Set([...myScopes.channels, ...myScopes.communities].map((s) => s.id)),
  );
  const needle = $derived(query.trim().toLowerCase());
  const listed = $derived([...scopes, ...myScopes.communities.filter((s) => s.isPrivate)]);
  // web lists channels then communities, or both by member count under "All".
  const filtered = $derived(
    (activeFilter === 'all'
      ? [...listed].sort((a, b) => b.members - a.members)
      : listed.filter((s) => s.kind === activeFilter)
    ).filter((s) => !needle || s.handle.toLowerCase().includes(needle) || s.slug.includes(needle)),
  );
  const emptyNoun = $derived(
    activeFilter === 'channel'
      ? 'channels'
      : activeFilter === 'community'
        ? 'communities'
        : 'channels or communities',
  );

  const memberLabel = (count: number): string => `${count} ${count === 1 ? 'member' : 'members'}`;

  let joining = $state<string | null>(null);
  let joinError = $state('');

  /** Join from the list (Plan, Phase and Feedback Screens, 6); the rail then lists it. */
  async function join(scope: Scope): Promise<void> {
    if (!requireViewer(`Sign in to join ${scope.handle}.`)) return;
    joining = scope.id;
    joinError = '';
    try {
      await joinProject(scope.id, true);
      showMyScopes(await loadMyScopes().catch(() => []));
    } catch (e) {
      joinError = explain(e);
    } finally {
      joining = null;
    }
  }

  function choose(): void {
    onNavigate();
    onClose();
  }
</script>

<OverlaySheet {open} title="Discover" rootClass="scope-discover" {onClose}>
  {#snippet toolbar()}
    <div class="discover-top">
      <label class="discover-field">
        <svg aria-hidden="true" viewBox="0 0 24 24"
          ><circle cx="11" cy="11" r="6" /><path d="m20 20-4.5-4.5" /></svg
        >
        <span class="sr-only">Search {emptyNoun}</span>
        <input
          id="scope-discover-search"
          bind:value={query}
          placeholder="Find channels and communities"
          type="search"
        />
      </label>
      <div class="discover-seg" role="radiogroup" aria-label="Filter channels and communities">
        {#each filters as filter (filter.value)}
          <button
            role="radio"
            aria-checked={activeFilter === filter.value}
            class:on={activeFilter === filter.value}
            type="button"
            onclick={() => (activeFilter = filter.value)}>{filter.label}</button
          >
        {/each}
      </div>
    </div>
  {/snippet}

  <div class="scope-list">
    {#if loading}
      <p class="empty-row">Loading…</p>
    {:else if loadError}
      <p class="empty-row">{loadError}</p>
    {:else if filtered.length === 0}
      <p class="empty-row">{query.trim() ? 'No matches.' : `No ${emptyNoun} yet.`}</p>
    {:else}
      {#if !needle}<p class="discover-label">Most members</p>{/if}
      {#each filtered as scope (scope.id)}
        {@const isJoined = joined.has(scope.id)}
        <div class="scope-row">
          <!-- eslint-disable-next-line svelte/no-navigation-without-resolve -- scopeHref resolves -->
          <a class="scope-row-link" href={scopeHref(scope)} onclick={choose}>
            <span class="scope-icon" data-kind={scope.kind}>
              {#if scope.pictureUrl}<img
                  class="discover-picture"
                  src={scope.pictureUrl}
                  alt=""
                />{:else if scope.kind === 'channel'}#{:else}<SurfaceIcon
                  icon="community"
                  size="sm"
                />{/if}
            </span>
            <span class="scope-copy">
              <strong>{scope.handle}</strong>
              <span class="scope-meta"
                >{scope.kind === 'channel' ? 'Channel' : 'Community'} · {memberLabel(
                  scope.members,
                )}{#if scope.isPrivate}
                  · Private{/if}</span
              >
            </span>
          </a>
          <button
            class="scope-join"
            class:joined={isJoined}
            type="button"
            disabled={isJoined || joining === scope.id}
            aria-label={isJoined ? `Joined ${scope.handle}` : `Join ${scope.handle}`}
            onclick={() => void join(scope)}
            >{isJoined ? 'Joined' : joining === scope.id ? 'Joining…' : 'Join'}</button
          >
        </div>
      {/each}
    {/if}
    {#if needle}
      <!-- eslint-disable svelte/no-navigation-without-resolve -- resolved paths -->
      <a class="discover-start" href={resolve('/create/channel')} onclick={choose}>
        <span class="scope-icon">+</span>
        <span class="scope-copy">
          <strong>Start "{query.trim()}" yourself</strong>
          <span class="scope-meta">Create a channel or a community</span>
        </span>
      </a>
      <!-- eslint-enable svelte/no-navigation-without-resolve -->
    {/if}
    {#if joinError}<p class="empty-row" role="alert">{joinError}</p>{/if}
  </div>
</OverlaySheet>

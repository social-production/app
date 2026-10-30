<script lang="ts">
  import IconMenuButton from '#lib/components/IconMenuButton.svelte';
  import SurfaceIcon from '#lib/components/icons/SurfaceIcon.svelte';
  import ToolbarIcon from '#lib/components/icons/ToolbarIcon.svelte';
  import OverlaySheet from '#lib/components/overlay/OverlaySheet.svelte';
  import { listScopes } from '#lib/services/scopes.ts';
  import { myScopes } from '#lib/stores/myScopes.svelte.ts';
  import type { Scope, ScopeKind } from '#lib/domain/scopes.ts';

  import { scopeHref } from './myScopesLoop.ts';

  import '#lib/components/styles/scopes.css';

  /** web's ScopeDiscoverSheet (left rail "Discover"): every channel and community, searchable. */
  interface Props {
    open: boolean;
    onClose: () => void;
    /** Called after a row is chosen (closes the phone rails). */
    onNavigate: () => void;
  }

  let { open, onClose, onNavigate }: Props = $props();

  type DiscoverFilter = 'all' | ScopeKind;

  const filters: { value: DiscoverFilter; label: string; icon?: ScopeKind }[] = [
    { value: 'all', label: 'All' },
    { value: 'channel', label: 'Channels', icon: 'channel' },
    { value: 'community', label: 'Communities', icon: 'community' },
  ];

  let query = $state('');
  let activeFilter = $state<DiscoverFilter>('all');
  let scopes = $state<Scope[]>([]);
  let loading = $state(false);
  let loadError = $state('');
  let requestId = 0;

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
      const found = await listScopes(needle, 200);
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
  // web lists channels then communities, or both by member count under "All".
  const filtered = $derived(
    (activeFilter === 'all'
      ? [...scopes].sort((a, b) => b.members - a.members)
      : scopes.filter((s) => s.kind === activeFilter)
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

  function choose(): void {
    onNavigate();
    onClose();
  }
</script>

<OverlaySheet {open} title="Discover" rootClass="scope-discover" {onClose}>
  {#snippet toolbar()}
    <div class="sheet-toolbar">
      <label class="sr-only" for="scope-discover-search">Search {emptyNoun}</label>
      <input
        id="scope-discover-search"
        bind:value={query}
        placeholder={`Search ${emptyNoun}`}
        type="search"
      />
      <IconMenuButton
        value={activeFilter}
        ariaLabel="Filter channels and communities"
        defaultValue="all"
        options={filters}
        showOptionIcons
        showTriggerLabel
        onChange={(next) => (activeFilter = next)}
      >
        <ToolbarIcon name="filter" />
      </IconMenuButton>
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
      {#each filtered as scope (scope.id)}
        <!-- eslint-disable-next-line svelte/no-navigation-without-resolve -- scopeHref resolves -->
        <a class="scope-row" href={scopeHref(scope)} onclick={choose}>
          <span class="scope-icon">
            <SurfaceIcon icon={scope.kind} size="sm" />
          </span>
          <span class="scope-copy">
            <strong>{scope.handle}</strong>
            <span class="scope-meta">{memberLabel(scope.members)}</span>
          </span>
          {#if joined.has(scope.id)}
            <span class="badge">Joined</span>
          {/if}
        </a>
      {/each}
    {/if}
  </div>
</OverlaySheet>

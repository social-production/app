<script lang="ts">
  import { resolve } from '$app/paths';
  import brandIcon from '#lib/assets/brand/app-icon-no-background-full-white.png';
  import GroupsIcon from '#lib/components/icons/GroupsIcon.svelte';
  import ToolbarIcon from '#lib/components/icons/ToolbarIcon.svelte';

  import SvgGlyph from './SvgGlyph.svelte';

  /** Top bar ported from web/src/lib/app/shell/AppShell.svelte; styles live in shell.css. */
  interface Props {
    compact: boolean;
    signedIn: boolean;
    viewerName: string;
    leftOpen: boolean;
    rightOpen: boolean;
    isActive: (href: string) => boolean;
    toggleLeft: () => void;
    toggleRight: () => void;
    onSearch: (query: string) => void;
    onSignIn: () => void;
  }

  let p: Props = $props();

  let searchExpanded = $state(false);
  let query = $state('');

  function submit(event: SubmitEvent): void {
    event.preventDefault();
    if (query.trim()) p.onSearch(query.trim());
  }
</script>

<header
  class="topbar"
  class:search-expanded={p.compact && searchExpanded}
  class:topbar-compact={p.compact}
>
  {#if p.compact && searchExpanded}
    <form class="toolbar-search toolbar-search-expanded" role="search" onsubmit={submit}>
      <button
        aria-label="Close search"
        title="Close search"
        class="search-close-button"
        type="button"
        onclick={() => (searchExpanded = false)}
      >
        <SvgGlyph name="close" className="search-icon" />
      </button>
      <div class="toolbar-search-wrap">
        <input
          aria-label="Search"
          bind:value={query}
          class="toolbar-search-input"
          placeholder="Search…"
          type="search"
        />
      </div>
    </form>
  {:else}
    <a class="brand" href={resolve('/')} aria-label="Social Production home">
      <span class="brand-mark"><img alt="" class="brand-icon" src={brandIcon} /></span>
      <span><strong>Social Production</strong></span>
    </a>

    <div class="panel-controls">
      <button
        aria-label="Open networks"
        title="Networks"
        aria-expanded={p.leftOpen}
        class="panel-toggle"
        data-active={p.leftOpen}
        type="button"
        onclick={p.toggleLeft}
      >
        <GroupsIcon className="panel-toggle-icon groups-icon" />
      </button>
      <button
        aria-label="Open map"
        title="Map (coming in M6)"
        class="panel-toggle panel-toggle-map"
        data-active={false}
        type="button"
        disabled
      >
        <SvgGlyph name="map" className="panel-toggle-icon" />
      </button>
      <button
        aria-label="Open activity"
        title="Activity"
        aria-expanded={p.rightOpen}
        class="panel-toggle panel-toggle-right"
        data-active={p.rightOpen}
        type="button"
        onclick={p.toggleRight}
      >
        <SvgGlyph name="activity" className="panel-toggle-icon" />
      </button>
    </div>

    <div class="toolbar-center">
      {#if p.compact}
        <button
          aria-label="Open search"
          title="Open search"
          class="search-open-button"
          type="button"
          onclick={() => (searchExpanded = true)}
        >
          <SvgGlyph name="search" className="search-icon" />
        </button>
      {:else}
        <form class="toolbar-search" role="search" onsubmit={submit}>
          <div class="toolbar-search-wrap">
            <input
              aria-label="Search"
              bind:value={query}
              class="toolbar-search-input"
              placeholder="Search…"
              type="search"
            />
          </div>
        </form>
        <nav class="primary-nav" aria-label="Primary">
          <a
            aria-label="Home"
            class:active-link={p.isActive('/')}
            class="nav-link nav-link-icon"
            href={resolve('/')}><ToolbarIcon name="home" /></a
          >
          <a
            aria-label="Notifications"
            title="Notifications"
            class:active-link={p.isActive('/notifications')}
            class="nav-link nav-link-icon"
            href={resolve('/notifications')}><ToolbarIcon name="bell" /></a
          >
          <a
            aria-label="Messages"
            title="Messages"
            class:active-link={p.isActive('/messages')}
            class="nav-link nav-link-icon"
            href={resolve('/messages')}><ToolbarIcon name="send" /></a
          >
        </nav>
      {/if}
    </div>

    <nav class="utility-nav" aria-label="Utilities">
      {#if !p.compact}
        <a class:active-link={p.isActive('/about')} class="utility-link" href={resolve('/about')}
          >About</a
        >
      {/if}
      {#if p.signedIn}
        {#if !p.compact}
          <a
            class:active-link={p.isActive('/profile')}
            class="utility-link"
            href={resolve('/profile')}>{p.viewerName}</a
          >
          <div class="settings-wrap">
            <a
              aria-label="Settings"
              title="Settings"
              class:active-link={p.isActive('/settings')}
              class="gear-button"
              href={resolve('/settings')}
            >
              <SvgGlyph name="gear" className="gear-icon" />
            </a>
          </div>
        {/if}
      {:else if !p.compact}
        <button class="utility-link" type="button" onclick={p.onSignIn}>Sign in / Sign up</button>
      {/if}
    </nav>
  {/if}
</header>

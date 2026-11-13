<script lang="ts">
  import type { ResolvedPathname } from '$app/types';
  import CountBadge from '#lib/components/icons/CountBadge.svelte';
  import ToolbarIcon from '#lib/components/icons/ToolbarIcon.svelte';

  interface Tab {
    href: ResolvedPathname;
    label: string;
    icon: string;
    badge?: number;
  }

  interface Props {
    tabs: Tab[];
    moreActive?: boolean;
    collapsed?: boolean;
    isActive: (href: string) => boolean;
    onMore: () => void;
    onNavigate?: () => void;
  }

  let {
    tabs,
    moreActive = false,
    collapsed = false,
    isActive,
    onMore,
    onNavigate,
  }: Props = $props();
</script>

<nav
  aria-label="Primary mobile"
  class="mobile-bottom-nav"
  class:chrome-collapsed={collapsed}
  aria-hidden={collapsed}
>
  <!-- eslint-disable svelte/no-navigation-without-resolve -- hrefs arrive resolved (ResolvedPathname) -->
  {#each tabs as tab (tab.href)}
    <a
      aria-label={tab.label}
      class:active-link={isActive(tab.href)}
      class="bottom-nav-item"
      href={tab.href}
      onclick={onNavigate}
    >
      <span class="bottom-nav-icon">
        <ToolbarIcon name={tab.icon} />
        {#if tab.badge && tab.badge > 0}
          <CountBadge count={tab.badge} />
        {/if}
      </span>
      {#if isActive(tab.href)}<span class="bottom-nav-label" aria-hidden="true">{tab.label}</span
        >{/if}
    </a>
    <!-- eslint-enable svelte/no-navigation-without-resolve -->
  {/each}

  <button
    aria-label="More"
    title="More"
    class="bottom-nav-item"
    class:active-link={moreActive}
    aria-expanded={moreActive}
    aria-haspopup="dialog"
    type="button"
    onclick={onMore}
  >
    <span class="bottom-nav-icon" aria-hidden="true">
      <ToolbarIcon name="more" />
    </span>
    {#if moreActive}<span class="bottom-nav-label" aria-hidden="true">More</span>{/if}
  </button>
</nav>

<style>
  /*
    ADR 0096: a floating capsule, not an edge-to-edge bar. The page scrolls under it and
    shows through the blur; the current tab is a tinted pill that names itself.
  */
  .mobile-bottom-nav {
    position: fixed;
    left: calc(var(--shell-nav-margin-x) + var(--shell-safe-left));
    right: calc(var(--shell-nav-margin-x) + var(--shell-safe-right));
    bottom: calc(var(--shell-nav-margin-bottom) + var(--shell-safe-bottom));
    z-index: var(--z-shell-nav);
    box-sizing: border-box;
    display: flex;
    gap: 4px;
    height: var(--shell-nav-capsule-height);
    margin: 0;
    padding: 6px;
    border: 1px solid var(--float-border);
    border-radius: 999px;
    background: var(--float-background);
    box-shadow: var(--float-shadow);
    -webkit-backdrop-filter: var(--float-blur);
    backdrop-filter: var(--float-blur);
    transition:
      transform 0.22s ease,
      visibility 0s;
    will-change: transform;
  }

  .mobile-bottom-nav.chrome-collapsed {
    transform: translateY(
      calc(100% + var(--shell-nav-margin-bottom) + var(--shell-safe-bottom) + 12px)
    );
    pointer-events: none;
    visibility: hidden;
    transition:
      transform 0.22s ease,
      visibility 0s linear 0.22s;
  }

  .bottom-nav-item {
    flex: 1 1 0;
    min-width: 0;
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 7px;
    min-height: 0;
    padding: 0;
    border: none;
    border-radius: 999px;
    background: transparent;
    color: var(--text-soft);
    font-size: 13.5px;
    font-weight: 700;
    line-height: 1;
    text-decoration: none;
    transition:
      color 0.16s ease,
      background-color 0.16s ease,
      transform 0.12s ease;
  }

  .bottom-nav-item.active-link {
    flex: 0 0 auto;
    padding: 0 16px;
  }

  .bottom-nav-item:active {
    transform: scale(0.94);
  }

  .bottom-nav-label {
    white-space: nowrap;
  }

  .bottom-nav-icon {
    position: relative;
    display: grid;
    place-items: center;
    width: 28px;
    height: 28px;
  }

  .bottom-nav-icon :global(.toolbar-icon) {
    width: 22px;
    height: 22px;
  }

  .bottom-nav-item.active-link {
    color: var(--brand-strong);
    background: var(--brand-soft);
    filter: none;
  }

  @media (hover: hover) and (pointer: fine) {
    .bottom-nav-item:hover {
      color: var(--brand-strong);
    }
  }

  .bottom-nav-icon :global(.count-badge) {
    position: absolute;
    top: -5px;
    right: -8px;
  }

  @media (prefers-reduced-motion: reduce) {
    .mobile-bottom-nav,
    .bottom-nav-item {
      transition: none;
    }
  }
</style>

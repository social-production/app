<script lang="ts">
  import type { Snippet } from 'svelte';

  import type { ResolvedPathname } from '$app/types';

  import { goto } from '$app/navigation';
  import { resolve } from '$app/paths';
  import { page } from '$app/state';

  import CreateFab, { type CreateLink } from '#lib/components/shell/CreateFab.svelte';
  import LeftRail from '#lib/components/shell/LeftRail.svelte';
  import MobileBottomNav from '#lib/components/shell/MobileBottomNav.svelte';
  import SignedOutIntro from '#lib/components/shell/SignedOutIntro.svelte';
  import Topbar from '#lib/components/shell/Topbar.svelte';
  import { connect } from '#lib/services/account.ts';
  import { getPerson } from '#lib/services/people.ts';
  import { chatChrome } from '#lib/stores/chatChrome.svelte.ts';
  import { connection } from '#lib/stores/connection.svelte.ts';
  import { viewer } from '#lib/stores/viewer.svelte.ts';

  import '#lib/components/shell/shell.css';
  import '#lib/components/styles/additions.css';

  /** The app frame, ported from web/src/lib/app/shell/AppShell.svelte (same classes and layout). */
  let { children }: { children: Snippet } = $props();

  let compact = $state(true);
  let leftOpen = $state(false);
  let rightOpen = $state(false);

  const signedIn = $derived(connection.current.status === 'connected');
  // "Sign in / Sign up" hides the intro banner; the sign-in card below it stays.
  const topbarHeight = $derived(compact ? 52 : 53);
  // web's immersiveChat: on phones an open detail chat hides the top bar and bottom nav.
  const immersive = $derived(compact && chatChrome.immersive);
  const shellTopbarHeight = $derived(immersive ? 0 : topbarHeight);
  const frameMaxWidth = $derived(!compact && !leftOpen && !rightOpen ? 'min(80%, 1200px)' : 'none');
  const bottomNav = $derived(compact && !immersive ? 'var(--shell-bottom-nav-height)' : '0px');
  const tabs: { href: ResolvedPathname; label: string; icon: string }[] = [
    { href: resolve('/'), label: 'Home', icon: 'home' },
    { href: resolve('/notifications'), label: 'Notifications', icon: 'bell' },
    { href: resolve('/messages'), label: 'Messages', icon: 'send' },
  ];

  $effect(() => {
    const media = window.matchMedia('(max-width: 1080px)');
    const apply = (): void => {
      compact = media.matches;
      leftOpen = !media.matches; // desktop opens the networks rail, as in web
      rightOpen = false;
    };
    apply();
    media.addEventListener('change', apply);
    return () => media.removeEventListener('change', apply);
  });

  // Connect in the background on every page; public pages work signed out (as on web).
  $effect(() => {
    void connect();
  });

  $effect(() => {
    const current = connection.current;
    if (current.status !== 'connected') return;
    void getPerson(current.agent).then((person) => {
      viewer.name = person.profile?.name ?? 'Profile';
    });
  });

  // As on web: the "+" sits on feed pages only (web's isFeedDiscoveryPath).
  const showCreateFab = $derived(['/', '/search'].includes(page.url.pathname));
  const createLinks: CreateLink[] = [
    {
      href: resolve('/create/project'),
      label: 'Project',
      description: 'Start a productive or service project',
      icon: 'project',
    },
  ];

  function isActive(href: string): boolean {
    const path = page.url.pathname;
    return href === '/' ? path === '/' : path.startsWith(href);
  }

  function closeCompact(): void {
    if (compact) leftOpen = rightOpen = false;
  }

  function toggleLeft(): void {
    leftOpen = !leftOpen;
    if (compact && leftOpen) rightOpen = false;
  }

  function toggleRight(): void {
    rightOpen = !rightOpen;
    if (compact && rightOpen) leftOpen = false;
  }
</script>

<div
  class="shell"
  data-signed-in={signedIn}
  class:shell-compact={compact}
  class:feed-chrome-collapsed={immersive}
  style={`--left-width: ${leftOpen && !compact ? '262px' : '0px'}; --right-width: ${rightOpen && !compact ? '292px' : '0px'}; --topbar-height: ${shellTopbarHeight}px; --topbar-natural-height: ${topbarHeight}px; --compact-content-offset: 0px; --shell-bottom-nav-offset: ${bottomNav}; --install-banner-offset: 0px; --shell-scroll-pad: ${bottomNav}; --shell-dock-safe-bottom: ${bottomNav === '0px' ? 'var(--shell-safe-bottom)' : '0px'}; --main-frame-max-width: ${frameMaxWidth};`}
>
  <Topbar
    {compact}
    collapsed={immersive}
    {signedIn}
    viewerName={viewer.name}
    {leftOpen}
    {rightOpen}
    {isActive}
    {toggleLeft}
    {toggleRight}
    onSearch={(q) => goto(resolve(`/search?q=${encodeURIComponent(q)}`))}
    onSignIn={() => goto(resolve('/onboarding'))}
  />

  {#if compact && (leftOpen || rightOpen)}
    <button
      aria-label="Close side panels"
      title="Close side panels"
      class="rail-backdrop"
      onclick={closeCompact}
    ></button>
  {/if}

  <div class="content-grid">
    <aside class="rail left-rail" data-open={leftOpen}>
      <LeftRail closePanels={closeCompact} onDiscover={() => goto(resolve('/'))} />
    </aside>

    <main class="main-content" class:main-content-compact={compact}>
      <div class="main-frame">
        {#if connection.current.status === 'signed-out' && page.url.pathname !== '/onboarding'}
          <SignedOutIntro
            onSignup={() => goto(resolve('/onboarding'))}
            aboutHref={resolve('/about')}
          />
        {/if}
        {@render children()}
      </div>
      {#if showCreateFab}
        <CreateFab {signedIn} links={createLinks} signInHref={resolve('/onboarding')} />
      {/if}
    </main>

    <aside class="rail right-rail" data-open={rightOpen}>
      {#if rightOpen}
        <section class="rail-panel">
          <p class="empty-hint">Your activity will appear here.</p>
        </section>
      {/if}
    </aside>
  </div>

  {#if compact && !immersive}
    <MobileBottomNav
      {tabs}
      {isActive}
      moreActive={leftOpen}
      onMore={toggleLeft}
      onNavigate={closeCompact}
    />
  {/if}
</div>

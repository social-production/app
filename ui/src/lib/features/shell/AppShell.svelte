<script lang="ts">
  import type { Snippet } from 'svelte';

  import type { ResolvedPathname } from '$app/types';

  import { goto } from '$app/navigation';
  import { resolve } from '$app/paths';
  import { page } from '$app/state';

  import LeftRail from '#lib/components/shell/LeftRail.svelte';
  import MobileBottomNav from '#lib/components/shell/MobileBottomNav.svelte';
  import SignedOutIntro from '#lib/components/shell/SignedOutIntro.svelte';
  import Topbar from '#lib/components/shell/Topbar.svelte';
  import AccountGate from '#lib/features/account/AccountGate.svelte';
  import { getPerson } from '#lib/services/people.ts';
  import { connection } from '#lib/stores/connection.svelte.ts';
  import { viewer } from '#lib/stores/viewer.svelte.ts';

  import '#lib/components/shell/shell.css';

  /** The app frame, ported from web/src/lib/app/shell/AppShell.svelte (same classes and layout). */
  let { children }: { children: Snippet } = $props();

  let compact = $state(true);
  let leftOpen = $state(false);
  let rightOpen = $state(false);
  let showWelcome = $state(false);

  const signedIn = $derived(connection.current.status === 'connected');
  // "Sign in / Sign up" hides the intro banner; the sign-in card below it stays.
  const topbarHeight = $derived(compact ? 52 : 53);
  const frameMaxWidth = $derived(!compact && !leftOpen && !rightOpen ? 'min(80%, 1200px)' : 'none');
  const bottomNav = $derived(compact ? 'var(--shell-bottom-nav-height)' : '0px');
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

  $effect(() => {
    const current = connection.current;
    if (current.status !== 'connected') return;
    void getPerson(current.agent).then((person) => {
      viewer.name = person.profile?.name ?? 'Profile';
    });
  });

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
  class:shell-compact={compact}
  style={`--left-width: ${leftOpen && !compact ? '262px' : '0px'}; --right-width: ${rightOpen && !compact ? '292px' : '0px'}; --topbar-height: ${topbarHeight}px; --topbar-natural-height: ${topbarHeight}px; --compact-content-offset: 0px; --shell-bottom-nav-offset: ${bottomNav}; --install-banner-offset: 0px; --shell-scroll-pad: ${bottomNav}; --shell-dock-safe-bottom: ${compact ? '0px' : 'var(--shell-safe-bottom)'}; --main-frame-max-width: ${frameMaxWidth};`}
>
  <Topbar
    {compact}
    {signedIn}
    viewerName={viewer.name}
    {leftOpen}
    {rightOpen}
    {isActive}
    {toggleLeft}
    {toggleRight}
    onSearch={(q) => goto(resolve(`/search?q=${encodeURIComponent(q)}`))}
    onSignIn={() => (showWelcome = true)}
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
        {#if !showWelcome && connection.current.status === 'signed-out'}
          <SignedOutIntro onSignup={() => (showWelcome = true)} aboutHref={resolve('/about')} />
        {/if}
        <AccountGate>{@render children()}</AccountGate>
      </div>
    </main>

    <aside class="rail right-rail" data-open={rightOpen}>
      {#if rightOpen}
        <section class="rail-panel">
          <p class="empty-hint">Your activity will appear here.</p>
        </section>
      {/if}
    </aside>
  </div>

  {#if compact}
    <MobileBottomNav
      {tabs}
      {isActive}
      moreActive={leftOpen}
      onMore={toggleLeft}
      onNavigate={closeCompact}
    />
  {/if}
</div>

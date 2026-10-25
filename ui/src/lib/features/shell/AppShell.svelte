<script lang="ts">
  import { untrack, type Snippet } from 'svelte';

  import type { ResolvedPathname } from '$app/types';

  import { goto } from '$app/navigation';
  import { resolve } from '$app/paths';
  import { page } from '$app/state';

  import CreateFab, { type CreateLink } from '#lib/components/shell/CreateFab.svelte';
  import LeftRail from '#lib/components/shell/LeftRail.svelte';
  import MobileBottomNav from '#lib/components/shell/MobileBottomNav.svelte';
  import MobileMoreSheet from '#lib/components/shell/MobileMoreSheet.svelte';
  import SignedOutIntro from '#lib/components/shell/SignedOutIntro.svelte';
  import Topbar from '#lib/components/shell/Topbar.svelte';
  import { startVisibilityPoll } from '#lib/components/actions/visibilityPoll.ts';
  import { connect } from '#lib/services/account.ts';
  import { myNotifications } from '#lib/services/notifications.ts';
  import { getPerson } from '#lib/services/people.ts';
  import { unreadMessageCount } from '#lib/services/unread.ts';
  import { applyAppearance, appearance } from '#lib/stores/appearance.svelte.ts';
  import { chatChrome } from '#lib/stores/chatChrome.svelte.ts';
  import { connection } from '#lib/stores/connection.svelte.ts';
  import { messageBadge } from '#lib/stores/messageBadge.svelte.ts';
  import { notifications, showNotifications } from '#lib/stores/notifications.svelte.ts';
  import { viewer } from '#lib/stores/viewer.svelte.ts';

  import AuthActionNotice from './AuthActionNotice.svelte';

  import '#lib/components/shell/shell.css';
  import '#lib/components/styles/additions.css';

  /** The app frame, ported from web/src/lib/app/shell/AppShell.svelte (same classes and layout). */
  let { children }: { children: Snippet } = $props();

  let compact = $state(true);
  let leftOpen = $state(false);
  let rightOpen = $state(false);
  let moreOpen = $state(false);

  const signedIn = $derived(connection.current.status === 'connected');
  // web's isAuthSurface: onboarding shows only the brand (no rails, bottom nav or "+").
  const authSurface = $derived(page.url.pathname === '/onboarding');
  // web shows the signed-out intro only on the feed, channels and communities.
  const showIntro = $derived(
    connection.current.status === 'signed-out' &&
      /^\/($|channels\/|communities\/)/.test(page.url.pathname),
  );
  // "Sign in / Sign up" hides the intro banner; the sign-in card below it stays.
  const topbarHeight = $derived(compact ? 52 : 53);
  // web's immersiveChat: on phones an open detail chat hides the top bar and bottom nav.
  const immersive = $derived(compact && chatChrome.immersive);
  const shellTopbarHeight = $derived(immersive ? 0 : topbarHeight);
  const frameMaxWidth = $derived(!compact && !leftOpen && !rightOpen ? 'min(80%, 1200px)' : 'none');
  const bottomNav = $derived(compact && !immersive ? 'var(--shell-bottom-nav-height)' : '0px');
  const tabs: { href: ResolvedPathname; label: string; icon: string; badge?: number }[] = $derived([
    { href: resolve('/'), label: 'Home', icon: 'home' },
    {
      href: resolve('/notifications'),
      label: 'Notifications',
      icon: 'bell',
      badge: notifications.unread,
    },
    { href: resolve('/messages'), label: 'Messages', icon: 'send', badge: messageBadge.unread },
  ]);

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

  // web's root layout: apply the appearance settings; "system" keeps following the device.
  $effect(() => {
    const query = window.matchMedia('(prefers-color-scheme: dark)');
    const apply = (): void => applyAppearance(query.matches);
    void [appearance.theme, appearance.accent, appearance.font, appearance.textSize];
    apply();
    query.addEventListener('change', apply);
    return () => query.removeEventListener('change', apply);
  });

  // Connect in the background on every page; public pages work signed out (as on web).
  $effect(() => {
    void connect();
  });

  // The bell's and the send icon's counts stay fresh while someone is signed in (every 30 s
  // while visible). The Messages page keeps the send icon's count itself, as on web.
  $effect(() => {
    if (!signedIn) return;
    const refresh = async (): Promise<void> => {
      const onMessages = page.url.pathname.startsWith('/messages');
      const [items, unread] = await Promise.allSettled([
        myNotifications(),
        onMessages ? null : unreadMessageCount(),
      ]);
      // A failed round keeps the old count; the next round tries again.
      if (items.status === 'fulfilled') showNotifications(items.value.items);
      if (unread.status === 'fulfilled' && unread.value !== null)
        messageBadge.unread = unread.value;
    };
    untrack(() => void refresh());
    const stop = startVisibilityPoll(refresh, { activeMs: 30_000 });
    return () => {
      stop();
      showNotifications([]);
      messageBadge.unread = 0;
    };
  });

  $effect(() => {
    const current = connection.current;
    if (current.status !== 'connected') return;
    void getPerson(current.agent).then((person) => {
      viewer.name = person.profile?.name ?? 'Profile';
    });
  });

  // web's isFeedDiscoveryPath: the "+" sits on the feed, platform and profile pages (and
  // channel/community feeds once they exist), never on onboarding.
  const showCreateFab = $derived(
    !authSurface &&
      (['/', '/platform', '/profile'].includes(page.url.pathname) ||
        /^\/(profile|channels|communities)\/[^/]+$/.test(page.url.pathname)),
  );
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

  /** web's toggleMoreSheet: the phone "More" tab opens its own sheet, closing the rails. */
  function toggleMore(): void {
    moreOpen = !moreOpen;
    if (moreOpen) leftOpen = rightOpen = false;
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
  style={`--left-width: ${leftOpen && !compact && !authSurface ? '262px' : '0px'}; --right-width: ${rightOpen && !compact && !authSurface ? '292px' : '0px'}; --topbar-height: ${shellTopbarHeight}px; --topbar-natural-height: ${topbarHeight}px; --compact-content-offset: 0px; --shell-bottom-nav-offset: ${bottomNav}; --install-banner-offset: 0px; --shell-scroll-pad: ${bottomNav}; --shell-dock-safe-bottom: ${bottomNav === '0px' ? 'var(--shell-safe-bottom)' : '0px'}; --main-frame-max-width: ${frameMaxWidth};`}
>
  <Topbar
    {compact}
    collapsed={immersive}
    {signedIn}
    viewerName={viewer.name}
    notificationCount={notifications.unread}
    messageCount={messageBadge.unread}
    {leftOpen}
    {rightOpen}
    {isActive}
    {toggleLeft}
    {toggleRight}
    onSearch={(q) => goto(resolve(`/search?q=${encodeURIComponent(q)}`))}
    onSignIn={() => goto(resolve('/onboarding'))}
    auth={authSurface}
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
    <aside class="rail left-rail" data-open={leftOpen && !authSurface}>
      {#if !authSurface}
        <LeftRail closePanels={closeCompact} onDiscover={() => goto(resolve('/'))} />
      {/if}
    </aside>

    <main class="main-content" class:main-content-compact={compact}>
      <div class="main-frame">
        {#if showIntro}
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

    <aside class="rail right-rail" data-open={rightOpen && !authSurface}>
      {#if rightOpen && !authSurface}
        <section class="rail-panel">
          <p class="empty-hint">Your activity will appear here.</p>
        </section>
      {/if}
    </aside>
  </div>

  {#if compact && !immersive && !authSurface}
    <MobileBottomNav
      {tabs}
      {isActive}
      moreActive={moreOpen}
      onMore={toggleMore}
      onNavigate={() => {
        closeCompact();
        moreOpen = false;
      }}
    />
    <MobileMoreSheet
      open={moreOpen}
      viewer={signedIn ? { name: viewer.name, href: resolve('/profile') } : null}
      signInHref={resolve('/onboarding')}
      settingsHref={resolve('/settings')}
      aboutHref={resolve('/about')}
      {isActive}
      onClose={() => (moreOpen = false)}
    />
  {/if}
  <AuthActionNotice />
</div>

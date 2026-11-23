<script lang="ts">
  import { t } from '#lib/i18n/index.ts';
  import {
    SCOPE_PICTURES,
    SPACE_PICTURES,
    type ScopePictureOf,
    type SpacePictureOf,
  } from '#lib/domain/scopes.ts';
  import { setContext, untrack, type Snippet } from 'svelte';

  import { goto } from '$app/navigation';
  import { resolve } from '$app/paths';
  import { page } from '$app/state';

  import CreateFab from '#lib/components/shell/CreateFab.svelte';
  import LeftRail from '#lib/components/shell/LeftRail.svelte';
  import MobileBottomNav from '#lib/components/shell/MobileBottomNav.svelte';
  import MobileMoreSheet from '#lib/components/shell/MobileMoreSheet.svelte';
  import Topbar from '#lib/components/shell/Topbar.svelte';
  import { connect } from '#lib/services/account.ts';
  import { getPerson } from '#lib/services/people.ts';
  import { applyAppearance, appearance, setAppearance } from '#lib/stores/appearance.svelte.ts';
  import { applyBackButtons } from '#lib/stores/backButtons.svelte.ts';
  import '#lib/stores/displayZone.svelte.ts';
  import { chatChrome } from '#lib/stores/chatChrome.svelte.ts';
  import { connection } from '#lib/stores/connection.svelte.ts';
  import { settleBootSplash } from './bootSplash.ts';
  import { messageBadge } from '#lib/stores/messageBadge.svelte.ts';
  import { myScopes } from '#lib/stores/myScopes.svelte.ts';
  import { openSearch } from '#lib/stores/searchOverlay.svelte.ts';
  import * as bell from '#lib/stores/notifications.svelte.ts';
  import { viewer } from '#lib/stores/viewer.svelte.ts';
  import { countActionable } from '#lib/domain/activityRail.ts';
  import { activityRail } from '#lib/stores/activityRail.svelte.ts';

  import { refreshActivityRail, startActivityRail } from './activityRailLoop.ts';
  import AuthActionNotice from './AuthActionNotice.svelte';
  import { startBell } from './bellLoop.ts';
  import { MapToggle } from './mapToggle.svelte.ts';
  import { createLinks, surfaceLinks } from './createLinks.ts';
  import { navTabs } from './navTabs.ts';
  import { refuseIfDeactivated, showDeactivatedReason } from './deactivated.ts';
  import { railLinks, startMyScopes } from './myScopesLoop.ts';
  import { ScopePictures } from './scopePictures.svelte.ts';
  import { provideShellContexts } from './shellContexts.ts';
  import RightRailPanel from './RightRailPanel.svelte';
  import ScopeDiscoverSheet from './ScopeDiscoverSheet.svelte';
  import { isFeedSurface, ScrollChrome } from './scrollChrome.svelte.ts';

  import '#lib/components/shell/shell.css';
  import '#lib/components/styles/additions.css';
  import '#lib/components/styles/ux/index.css';

  /** The app frame, ported from web/src/lib/app/shell/AppShell.svelte (same classes and layout). */
  let { children, mapPanel }: { children: Snippet; mapPanel?: Snippet<[() => void]> } = $props();
  // Tags show their channel's or community's picture (ADR 0109).
  const pictures = new ScopePictures();
  setContext<ScopePictureOf>(SCOPE_PICTURES, pictures.of);
  setContext<SpacePictureOf>(SPACE_PICTURES, pictures.ofHash);
  provideShellContexts();

  let compact = $state(true);
  let leftOpen = $state(false);
  let rightOpen = $state(false);
  let moreOpen = $state(false);
  let discover = $state({ open: false, query: '' });
  // web's map panel over the content, or the `/map` page (mapToggle.ts, ADR 0085).
  const mapPage = $derived(page.url.pathname === '/map');
  const map = new MapToggle(
    () => ({ left: leftOpen, right: rightOpen, compact }),
    (r) => ([leftOpen, rightOpen] = [r.left, r.right]),
    () => ({ path: page.url.pathname, mapPage }),
  );

  const signedIn = $derived(connection.current.status === 'connected');
  // web's isAuthSurface: onboarding shows only the brand (no rails, bottom nav or "+").
  const authSurface = $derived(page.url.pathname === '/onboarding');
  const topbarHeight = $derived(compact ? 52 : 53);
  // web's immersiveChat: on phones an open detail chat hides the top bar and bottom nav.
  const immersive = $derived(compact && chatChrome.immersive);
  // Sign-up has its own logo, so the top bar goes as it does for an open chat.
  const shellTopbarHeight = $derived(immersive || authSurface ? 0 : topbarHeight);
  const frameMaxWidth = $derived(
    !compact && !leftOpen && !rightOpen ? t('min(80%, 1200px)') : 'none',
  );
  const bottomNav = $derived(compact && !immersive ? 'var(--shell-bottom-nav-height)' : '0px');
  // Known to be signed out (not still connecting): no Notifications or Messages to offer.
  const guest = $derived(connection.current.status === 'signed-out');
  const tabs = $derived(navTabs(guest, bell.notifications.unread, messageBadge.unread));

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

  // web's root layout: apply the appearance settings (and fix 3f's back buttons, read in apply).
  $effect(() => {
    const query = window.matchMedia('(prefers-color-scheme: dark)');
    const apply = (): void => (applyAppearance(query.matches), applyBackButtons());
    void [appearance.theme, appearance.accent, appearance.font, appearance.textSize];
    apply();
    query.addEventListener('change', apply);
    return () => query.removeEventListener('change', apply);
  });

  // Connect in the background on every page; public pages work signed out (as on web).
  $effect(() => {
    void connect();
  });

  $effect(() => {
    if (signedIn) return untrack(() => startBell(() => page.url.pathname.startsWith('/messages')));
  });

  // web's left rail lists the channels and communities you joined (ADR 0068).
  $effect(() => {
    if (signedIn) return untrack(() => startMyScopes());
  });

  // web's activity rail: its badge counts cards you haven't opened or cleared, and the first
  // desktop load opens it when there is something to do (until you toggle it yourself).
  $effect(() => {
    const current = connection.current;
    if (current.status !== 'connected') return;
    return untrack(() => startActivityRail(current.agent));
  });
  const railCount = $derived(
    countActionable(activityRail.items, activityRail.dismissed, activityRail.seen),
  );
  let railPreferenceApplied = false;
  $effect(() => {
    if (railPreferenceApplied || compact || !activityRail.loaded) return;
    railPreferenceApplied = true;
    if (railCount > 0 && !authSurface && !page.url.pathname.startsWith('/messages')) {
      untrack(() => (rightOpen = true));
    }
  });

  $effect(() => settleBootSplash(connection.current.status));
  $effect(showDeactivatedReason);
  $effect(() => {
    const current = connection.current;
    if (current.status !== 'connected') return;
    void getPerson(current.agent).then((person) => {
      viewer.name = person.profile?.name ?? t('Profile');
    });
    void refuseIfDeactivated(current.agent);
  });

  // web's isFeedDiscoveryPath: the "+" sits on feeds, never on onboarding or over the map.
  const feed = $derived(!authSurface && !map.open && isFeedSurface(page.url.pathname));
  const showCreateFab = $derived(feed && !page.url.pathname.startsWith('/profile')); // not on profiles (mockups §2)
  // Phone feeds slide the bars away while you scroll down (scrollChrome.svelte.ts).
  const scroll = new ScrollChrome(() => ({
    on: compact && feed && !immersive,
    path: page.url.pathname,
  }));
  const slid = $derived(scroll.hidden && !leftOpen && !rightOpen && !moreOpen);

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
    railPreferenceApplied = true;
    if (compact && rightOpen) leftOpen = false;
    if (rightOpen) refreshActivityRail();
  }
</script>

<div
  class="shell"
  data-signed-in={signedIn}
  class:shell-compact={compact}
  class:shell-map-open={map.open || mapPage}
  class:shell-map-page={mapPage}
  class:feed-chrome-collapsed={immersive}
  class:shell-chrome-slid={slid}
  style={`--left-width: ${leftOpen && !compact && !authSurface ? '262px' : '0px'}; --right-width: ${rightOpen && !compact && !authSurface ? '292px' : '0px'}; --topbar-height: ${shellTopbarHeight}px; --topbar-natural-height: ${topbarHeight}px; --compact-content-offset: 0px; --shell-bottom-nav-offset: ${bottomNav}; --install-banner-offset: 0px; --shell-scroll-pad: ${bottomNav}; --shell-dock-safe-bottom: ${bottomNav === '0px' ? 'var(--shell-safe-bottom)' : '0px'}; --main-frame-max-width: ${frameMaxWidth};`}
>
  <Topbar
    {compact}
    collapsed={immersive || authSurface || slid}
    {signedIn}
    {guest}
    viewerName={viewer.name}
    notificationCount={bell.notifications.unread}
    messageCount={messageBadge.unread}
    activityCount={railCount}
    {leftOpen}
    {rightOpen}
    {isActive}
    {toggleLeft}
    {toggleRight}
    mapOpen={map.open}
    toggleMap={() => map.toggle()}
    onSearch={openSearch}
    onSignIn={() => goto(resolve('/onboarding'))}
    auth={authSurface}
  />

  {#if compact && (leftOpen || rightOpen)}
    <button
      aria-label={t('Close side panels')}
      title={t('Close side panels')}
      class="rail-backdrop"
      onclick={closeCompact}
    ></button>
  {/if}

  <section class="map-overlay" class:map-overlay--open={map.open} aria-hidden={!map.open}>
    {#if map.open}{@render mapPanel?.(() => map.close())}{/if}
  </section>

  <div class="content-grid" class:content-grid--map-open={map.open || mapPage}>
    <aside class="rail left-rail" data-open={leftOpen && !authSurface}>
      {#if !authSurface}
        <LeftRail
          channels={railLinks(myScopes.channels)}
          communities={railLinks(myScopes.communities)}
          invites={bell.notifications.invites}
          closePanels={closeCompact}
          onDiscover={(query) => (discover = { open: true, query })}
        />
      {/if}
    </aside>

    <main class="main-content" class:main-content-compact={compact}>
      <div class="main-frame">
        {@render children()}
      </div>
      {#if showCreateFab}
        <CreateFab
          {signedIn}
          links={createLinks}
          surfaces={surfaceLinks}
          signInHref={resolve('/onboarding')}
          hidden={slid}
        />
      {/if}
    </main>

    <aside class="rail right-rail" data-open={rightOpen && !authSurface}>
      {#if rightOpen && !authSurface}
        <RightRailPanel onClose={closeCompact} />
      {/if}
    </aside>
  </div>

  {#if compact && !immersive && !authSurface}
    <MobileBottomNav
      {tabs}
      {isActive}
      moreActive={moreOpen}
      collapsed={slid}
      signUpHref={guest ? resolve('/onboarding') : null}
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
      theme={appearance.theme}
      onTheme={(theme) => setAppearance({ theme })}
      onClose={() => (moreOpen = false)}
    />
  {/if}
  <ScopeDiscoverSheet
    open={discover.open}
    initialQuery={discover.query}
    onClose={() => (discover.open = false)}
    onNavigate={closeCompact}
  />
  <AuthActionNotice />
</div>

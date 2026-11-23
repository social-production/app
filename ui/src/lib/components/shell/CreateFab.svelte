<script lang="ts" module>
  import type { SurfaceIconId } from '#lib/components/icons/surface.ts';

  import type { ResolvedPathname } from '$app/types';

  export interface CreateLink {
    href: ResolvedPathname;
    label: string;
    description: string;
    icon: SurfaceIconId;
    /** A short tag after the label ("NEW"); the item is tinted. */
    badge?: string;
  }
</script>

<script lang="ts">
  import { t } from '#lib/i18n/index.ts';
  import SurfaceIcon from '#lib/components/icons/SurfaceIcon.svelte';
  import SignInToCreate from './SignInToCreate.svelte';
  import '#lib/components/styles/fab.css';

  /**
   * web's CreateFab: the floating "+" on feed pages, opening the create menu. Items appear
   * as their milestones land (projects now; posts, threads, events, help requests later).
   */
  interface Props {
    signedIn: boolean;
    links: CreateLink[];
    /** web's "Surfaces" section: Community, Channel (ADR 0068). */
    surfaces?: CreateLink[];
    signInHref: ResolvedPathname;
    /** The feed is scrolling down: the + slides away with the bars (owner, 2026-11-23). */
    hidden?: boolean;
  }

  let { signedIn, links, surfaces = [], signInHref, hidden = false }: Props = $props();

  let open = $state(false);
  let asking = $state(false);

  $effect(() => {
    if (!open) return;
    const escape = (e: KeyboardEvent): void => {
      if (e.key === 'Escape') {
        e.preventDefault();
        open = false;
      }
    };
    window.addEventListener('keydown', escape);
    return () => window.removeEventListener('keydown', escape);
  });
</script>

<!-- eslint-disable svelte/no-navigation-without-resolve -- hrefs arrive resolved (ResolvedPathname) -->
<div class="create-fab" class:slid={hidden && !open}>
  {#if !signedIn}
    <div class="create-fab-shell">
      <button
        class="fab-button"
        type="button"
        aria-label={t('Sign in to create')}
        onclick={() => (asking = true)}
      >
        <svg aria-hidden="true" viewBox="0 0 24 24" class="fab-icon"
          ><path
            d="M12 5v14M5 12h14"
            stroke="currentColor"
            stroke-width="2.2"
            stroke-linecap="round"
            fill="none"
          /></svg
        >
      </button>
    </div>
    <SignInToCreate open={asking} {signInHref} onClose={() => (asking = false)} />
  {:else}
    {#if open}
      <button
        aria-label={t('Close create menu')}
        title={t('Close create menu')}
        class="fab-backdrop"
        type="button"
        onclick={() => (open = false)}
      ></button>
    {/if}
    {#snippet item(link: CreateLink)}
      <a
        class="create-menu-item"
        class:badged={!!link.badge}
        href={link.href}
        role="menuitem"
        onclick={() => (open = false)}
      >
        <span class="create-menu-icon-wrap"
          ><SurfaceIcon icon={link.icon} className="create-menu-icon" /></span
        >
        <span class="item-copy">
          <span class="item-label"
            >{t(link.label)}{#if link.badge}<small class="item-badge">{link.badge}</small
              >{/if}</span
          >
          <span class="item-description">{link.description}</span>
        </span>
      </a>
    {/snippet}
    <div class="create-fab-shell" class:menu-open={open}>
      {#if open}
        <div class="create-menu" role="menu" aria-label={t('Create')}>
          <div class="create-menu-body">
            <p class="menu-kicker">{t('Content')}</p>
            {#each links as link (link.href)}{@render item(link)}{/each}
            {#if surfaces.length > 0}
              <p class="menu-kicker menu-kicker-separated">{t('Surfaces')}</p>
              {#each surfaces as link (link.href)}{@render item(link)}{/each}
            {/if}
          </div>
          <button
            aria-label={t('Close')}
            class="menu-close sheet-close"
            type="button"
            onclick={() => (open = false)}>×</button
          >
        </div>
      {/if}
      <button
        aria-expanded={open}
        aria-haspopup="menu"
        aria-label={open ? t('Close create menu') : t('Create')}
        title={open ? t('Close create menu') : t('Create')}
        class="fab-button"
        class:fab-button-open={open}
        type="button"
        onclick={() => (open = !open)}
      >
        <svg aria-hidden="true" viewBox="0 0 24 24" class="fab-icon">
          {#if open}
            <path
              d="M8.5 8.5 15.5 15.5M15.5 8.5 8.5 15.5"
              stroke="currentColor"
              stroke-width="2.2"
              stroke-linecap="round"
              fill="none"
            />
          {:else}
            <path
              d="M12 5v14M5 12h14"
              stroke="currentColor"
              stroke-width="2.2"
              stroke-linecap="round"
              fill="none"
            />
          {/if}
        </svg>
      </button>
    </div>
  {/if}
</div>
<!-- eslint-enable svelte/no-navigation-without-resolve -->

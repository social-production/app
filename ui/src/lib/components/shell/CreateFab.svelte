<script lang="ts" module>
  import type { SurfaceIconId } from '#lib/components/icons/surface.ts';

  import type { ResolvedPathname } from '$app/types';

  export interface CreateLink {
    href: ResolvedPathname;
    label: string;
    description: string;
    icon: SurfaceIconId;
  }
</script>

<script lang="ts">
  import SurfaceIcon from '#lib/components/icons/SurfaceIcon.svelte';
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
  }

  let { signedIn, links, surfaces = [], signInHref }: Props = $props();

  let open = $state(false);

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
<div class="create-fab">
  {#if !signedIn}
    <div class="create-fab-shell">
      <a class="fab-button" href={signInHref} aria-label="Sign in to create">
        <svg aria-hidden="true" viewBox="0 0 24 24" class="fab-icon"
          ><path
            d="M12 5v14M5 12h14"
            stroke="currentColor"
            stroke-width="2.2"
            stroke-linecap="round"
            fill="none"
          /></svg
        >
      </a>
    </div>
  {:else}
    {#if open}
      <button
        aria-label="Close create menu"
        title="Close create menu"
        class="fab-backdrop"
        type="button"
        onclick={() => (open = false)}
      ></button>
    {/if}
    {#snippet item(link: CreateLink)}
      <a class="create-menu-item" href={link.href} role="menuitem" onclick={() => (open = false)}>
        <span class="create-menu-icon-wrap"
          ><SurfaceIcon icon={link.icon} className="create-menu-icon" /></span
        >
        <span class="item-copy">
          <span class="item-label">{link.label}</span>
          <span class="item-description">{link.description}</span>
        </span>
      </a>
    {/snippet}
    <div class="create-fab-shell" class:menu-open={open}>
      {#if open}
        <div class="create-menu" role="menu" aria-label="Create">
          <div class="create-menu-body">
            <p class="menu-kicker">Content</p>
            {#each links as link (link.href)}{@render item(link)}{/each}
            {#if surfaces.length > 0}
              <p class="menu-kicker menu-kicker-separated">Surfaces</p>
              {#each surfaces as link (link.href)}{@render item(link)}{/each}
            {/if}
          </div>
          <button
            aria-label="Close"
            class="menu-close sheet-close"
            type="button"
            onclick={() => (open = false)}>×</button
          >
        </div>
      {/if}
      <button
        aria-expanded={open}
        aria-haspopup="menu"
        aria-label={open ? 'Close create menu' : 'Create'}
        title={open ? 'Close create menu' : 'Create'}
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

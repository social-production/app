<script lang="ts">
  import type { ResolvedPathname } from '$app/types';

  import AvatarBadge from '#lib/components/icons/AvatarBadge.svelte';
  import '#lib/components/styles/ux/menus.css';

  /** The phone "More" tab's menu (ADR 0097): a floating panel above the nav capsule with
   * web's MobileMoreSheet links (profile or sign in, Settings, About) and the theme. */
  type Theme = 'system' | 'light' | 'dark';
  const THEMES: [Theme, string][] = [
    ['system', 'System'],
    ['light', 'Light'],
    ['dark', 'Dark'],
  ];

  interface Props {
    open: boolean;
    /** Signed in: the viewer's name and own profile; otherwise "Sign in / Sign up". */
    viewer: { name: string; href: ResolvedPathname } | null;
    signInHref: ResolvedPathname;
    settingsHref: ResolvedPathname;
    aboutHref: ResolvedPathname;
    isActive: (href: string) => boolean;
    theme: Theme;
    onTheme: (theme: Theme) => void;
    onClose: () => void;
  }

  let {
    open,
    viewer,
    signInHref,
    settingsHref,
    aboutHref,
    isActive,
    theme,
    onTheme,
    onClose,
  }: Props = $props();

  $effect(() => {
    const keydown = (event: KeyboardEvent): void => {
      if (event.key === 'Escape' && open) {
        event.preventDefault();
        onClose();
      }
    };
    window.addEventListener('keydown', keydown);
    return () => window.removeEventListener('keydown', keydown);
  });
</script>

{#if open}
  <div class="more-panel-root">
    <button
      aria-label="Close menu"
      title="Close menu"
      class="more-panel-backdrop"
      type="button"
      onclick={onClose}
    ></button>

    <div class="more-panel float-panel" role="dialog" aria-modal="true" aria-label="More menu">
      <!-- eslint-disable svelte/no-navigation-without-resolve -- hrefs arrive resolved (ResolvedPathname) -->
      {#if viewer}
        <a
          class:active-link={isActive(viewer.href)}
          class="more-panel-me"
          href={viewer.href}
          aria-label={viewer.name}
          onclick={onClose}
        >
          <AvatarBadge name={viewer.name} />
          <span class="more-panel-me-copy">
            <strong>{viewer.name}</strong>
            <span>View profile</span>
          </span>
        </a>
        <div class="float-panel-sep"></div>
        <a
          class:active-link={isActive(settingsHref)}
          class="float-panel-item"
          href={settingsHref}
          onclick={onClose}
        >
          <svg aria-hidden="true" viewBox="0 0 24 24"
            ><circle
              cx="12"
              cy="12"
              r="3"
              fill="none"
              stroke="currentColor"
              stroke-width="1.8"
            /><path
              d="M19.4 15a1.7 1.7 0 0 0 .3 1.8l.1.1a2 2 0 1 1-2.8 2.8l-.1-.1a1.7 1.7 0 0 0-1.8-.3 1.7 1.7 0 0 0-1 1.5V21a2 2 0 1 1-4 0v-.1a1.7 1.7 0 0 0-1.1-1.5 1.7 1.7 0 0 0-1.8.3l-.1.1a2 2 0 1 1-2.8-2.8l.1-.1a1.7 1.7 0 0 0 .3-1.8 1.7 1.7 0 0 0-1.5-1H3a2 2 0 1 1 0-4h.1a1.7 1.7 0 0 0 1.5-1.1 1.7 1.7 0 0 0-.3-1.8l-.1-.1a2 2 0 1 1 2.8-2.8l.1.1a1.7 1.7 0 0 0 1.8.3H9a1.7 1.7 0 0 0 1-1.5V3a2 2 0 1 1 4 0v.1a1.7 1.7 0 0 0 1 1.5 1.7 1.7 0 0 0 1.8-.3l.1-.1a2 2 0 1 1 2.8 2.8l-.1.1a1.7 1.7 0 0 0-.3 1.8V9c.3.6.9 1 1.5 1H21a2 2 0 1 1 0 4h-.1a1.7 1.7 0 0 0-1.5 1Z"
              fill="none"
              stroke="currentColor"
              stroke-width="1.8"
              stroke-linejoin="round"
            /></svg
          >
          <span>Settings</span>
        </a>
      {:else}
        <a class="float-panel-item" href={signInHref} onclick={onClose}>
          <svg aria-hidden="true" viewBox="0 0 24 24"
            ><circle
              cx="12"
              cy="8"
              r="3.2"
              fill="none"
              stroke="currentColor"
              stroke-width="1.8"
            /><path
              d="M6.2 18.5c1.2-2.4 3.2-3.5 5.8-3.5s4.6 1.1 5.8 3.5"
              fill="none"
              stroke="currentColor"
              stroke-width="1.8"
              stroke-linecap="round"
            /></svg
          >
          <span>Sign in / Sign up</span>
        </a>
      {/if}
      <a
        class:active-link={isActive(aboutHref)}
        class="float-panel-item"
        href={aboutHref}
        onclick={onClose}
      >
        <svg aria-hidden="true" viewBox="0 0 24 24"
          ><circle
            cx="12"
            cy="12"
            r="8"
            fill="none"
            stroke="currentColor"
            stroke-width="1.8"
          /><path
            d="M12 11v5"
            fill="none"
            stroke="currentColor"
            stroke-width="1.8"
            stroke-linecap="round"
          /><circle cx="12" cy="8" r="0.9" fill="currentColor" /></svg
        >
        <span>About</span>
      </a>
      <!-- eslint-enable svelte/no-navigation-without-resolve -->
      <div class="float-panel-sep"></div>
      <div class="more-panel-theme" role="radiogroup" aria-label="Theme">
        {#each THEMES as [value, label] (value)}
          <button
            type="button"
            role="radio"
            aria-checked={theme === value}
            class:on={theme === value}
            onclick={() => onTheme(value)}>{label}</button
          >
        {/each}
      </div>
    </div>
  </div>
{/if}

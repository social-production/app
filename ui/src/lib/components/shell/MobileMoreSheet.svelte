<script lang="ts">
  import type { ResolvedPathname } from '$app/types';

  import './more-sheet.css';

  /** web's MobileMoreSheet: the phone "More" tab's full-screen menu (profile or sign in,
   * Settings, About). The left rail stays on the top bar's button, as on web. */
  interface Props {
    open: boolean;
    /** Signed in: the viewer's name and own profile; otherwise "Sign in / Sign up". */
    viewer: { name: string; href: ResolvedPathname } | null;
    signInHref: ResolvedPathname;
    settingsHref: ResolvedPathname;
    aboutHref: ResolvedPathname;
    isActive: (href: string) => boolean;
    onClose: () => void;
  }

  let { open, viewer, signInHref, settingsHref, aboutHref, isActive, onClose }: Props = $props();

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
  <div class="more-sheet-root">
    <button
      aria-label="Close menu"
      title="Close menu"
      class="sheet-backdrop"
      type="button"
      onclick={onClose}
    ></button>

    <div class="more-sheet" role="dialog" aria-modal="true" aria-label="More menu">
      <button aria-label="Close" class="sheet-x" type="button" onclick={onClose}>×</button>
      <!-- eslint-disable svelte/no-navigation-without-resolve -- hrefs arrive resolved (ResolvedPathname) -->
      <div class="sheet-links">
        <a
          class:active-link={!!viewer && isActive(viewer.href)}
          class="sheet-link"
          href={viewer?.href ?? signInHref}
          onclick={onClose}
        >
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
          <span>{viewer?.name ?? 'Sign in / Sign up'}</span>
        </a>

        {#if viewer}
          <a
            class:active-link={isActive(settingsHref)}
            class="sheet-link"
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
        {/if}

        <a
          class:active-link={isActive(aboutHref)}
          class="sheet-link"
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
      </div>
    </div>
  </div>
{/if}

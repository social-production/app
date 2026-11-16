<script lang="ts">
  import type { ResolvedPathname } from '$app/types';

  import { resolve } from '$app/paths';
  import { page } from '$app/state';
  import ToolbarIcon from '#lib/components/icons/ToolbarIcon.svelte';

  import RailLinkRow from './RailLinkRow.svelte';

  interface RailLink {
    href: ResolvedPathname;
    label: string;
    imageUrl?: string | null;
  }

  interface Props {
    channels?: RailLink[];
    communities?: RailLink[];
    /** Invites waiting for an answer (web's unreadCounts.invites). */
    invites?: number;
    closePanels?: () => void;
    /** Opens Discover, with the words typed into the rail's field. */
    onDiscover?: (query: string) => void;
  }

  /** web's LeftRailPanel in the UX look (Plan, Phase and Feedback Screens, 5): a field that
   * opens Discover with your words, Platform and Moderators, folding Channels and
   * Communities with soft glyphs (the open one tinted), and Discover beside Feedback. */
  let { channels = [], communities = [], invites = 0, closePanels, onDiscover }: Props = $props();

  let find = $state('');
  const here = (href: string): boolean =>
    page.url.pathname + page.url.search === href || page.url.pathname === href;
  let channelsOpen = $state(true);
  let communitiesOpen = $state(true);
</script>

<div class="rail-ux">
  <form
    class="rail-find"
    role="search"
    onsubmit={(e) => {
      e.preventDefault();
      onDiscover?.(find.trim());
      find = '';
    }}
  >
    <svg aria-hidden="true" viewBox="0 0 24 24"
      ><circle cx="11" cy="11" r="6" /><path d="m20 20-4.5-4.5" /></svg
    >
    <input
      bind:value={find}
      type="search"
      aria-label="Find channels and communities"
      placeholder="Find channels and communities"
    />
  </form>

  <div class="stack-links">
    <RailLinkRow
      href={resolve('/platform')}
      icon="platform"
      label="Platform"
      active={here('/platform')}
      onclick={closePanels}
    />
    <RailLinkRow
      href={resolve('/platform?board=1')}
      icon="shield"
      label="Moderators"
      onclick={closePanels}
    />
  </div>

  <details class="rail-section" bind:open={channelsOpen}>
    <summary class="rail-section-summary">
      <span>Channels</span>
      <span class="chevron" aria-hidden="true"></span>
    </summary>
    <div class="stack-links">
      {#each channels as link (link.href)}
        <RailLinkRow
          href={link.href}
          icon="channel"
          imageUrl={link.imageUrl ?? null}
          label={link.label}
          active={here(link.href)}
          onclick={closePanels}
        />
      {:else}
        <p class="empty-hint">Join channels to see them here.</p>
      {/each}
    </div>
  </details>

  <details class="rail-section" bind:open={communitiesOpen}>
    <summary class="rail-section-summary">
      <span>Communities</span>
      <span class="chevron" aria-hidden="true"></span>
    </summary>
    <div class="stack-links">
      {#if invites > 0}
        <!-- Invites to private communities and events wait here until answered. -->
        <RailLinkRow
          href={resolve('/invites')}
          icon="community"
          label="Invites"
          active={here('/invites')}
          onclick={closePanels}
        >
          {#snippet trailing()}<span class="invite-count">{invites}</span>{/snippet}
        </RailLinkRow>
      {/if}
      {#each communities as link (link.href)}
        <RailLinkRow
          href={link.href}
          icon="community"
          imageUrl={link.imageUrl ?? null}
          label={link.label}
          active={here(link.href)}
          onclick={closePanels}
        />
      {:else}
        <p class="empty-hint">Join communities to see them here.</p>
      {/each}
    </div>
  </details>

  <div class="action-stack">
    <button class="rail-action discover-button" type="button" onclick={() => onDiscover?.('')}>
      <ToolbarIcon name="plus" />
      <span>Discover</span>
    </button>
    <a class="rail-action feedback-button" href={resolve('/feedback')} onclick={closePanels}>
      <ToolbarIcon name="pencil" />
      <span>Feedback</span>
    </a>
  </div>
</div>

<style>
  .rail-ux {
    display: grid;
    gap: 6px;
  }

  .rail-find {
    display: flex;
    align-items: center;
    gap: 8px;
    height: 42px;
    margin-bottom: 6px;
    padding: 0 14px;
    border-radius: 999px;
    background: var(--panel-hover);
    color: var(--text-soft);
  }

  .rail-find:focus-within {
    box-shadow: 0 0 0 2px color-mix(in srgb, var(--brand) 55%, transparent);
  }

  .rail-find svg {
    flex: none;
    width: 18px;
    height: 18px;
    fill: none;
    stroke: currentColor;
    stroke-width: 1.8;
    stroke-linecap: round;
  }

  .rail-find input {
    flex: 1;
    min-width: 0;
    height: 100%;
    padding: 0;
    border: none;
    outline: none;
    background: transparent;
    color: var(--text-main);
    font: inherit;
    font-size: 14.5px;
  }

  .invite-count {
    min-width: 20px;
    height: 20px;
    padding: 0 6px;
    border-radius: 999px;
    background: var(--brand);
    color: var(--on-brand, #fff);
    font-size: 11px;
    font-weight: 700;
    line-height: 20px;
    text-align: center;
  }

  .rail-section {
    display: grid;
  }

  .rail-section-summary {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 12px 8px 4px;
    color: var(--text-soft);
    font-size: 12.5px;
    font-weight: 700;
    list-style: none;
    cursor: pointer;
  }

  .rail-section-summary::-webkit-details-marker {
    display: none;
  }

  .chevron {
    width: 8px;
    height: 8px;
    margin-right: 4px;
    border-right: 2px solid currentColor;
    border-bottom: 2px solid currentColor;
    transform: rotate(-45deg);
    transition: transform 0.16s ease;
  }

  details[open] > .rail-section-summary .chevron {
    transform: rotate(45deg);
  }

  .stack-links {
    display: grid;
    gap: 2px;
  }

  .action-stack {
    display: flex;
    gap: 8px;
    padding: 12px 4px;
  }

  .rail-action {
    display: inline-flex;
    flex: 1;
    align-items: center;
    justify-content: center;
    gap: 7px;
    height: 42px;
    padding: 0 14px;
    border: none;
    border-radius: 999px;
    background: var(--panel-hover);
    color: var(--text-main);
    font: inherit;
    font-size: 14px;
    font-weight: 700;
    text-decoration: none;
    cursor: pointer;
  }

  .discover-button {
    background: var(--brand);
    color: var(--on-brand);
  }

  .empty-hint {
    margin: 0;
    padding: 4px 10px;
    color: var(--text-soft);
    font-size: 12.5px;
  }
</style>

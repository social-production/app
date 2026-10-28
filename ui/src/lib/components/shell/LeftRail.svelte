<script lang="ts">
  import type { ResolvedPathname } from '$app/types';

  import { resolve } from '$app/paths';
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
    onDiscover?: () => void;
  }

  /** Ported from web/src/lib/features/left-rail/LeftRailPanel.svelte (same markup and styles). */
  let { channels = [], communities = [], invites = 0, closePanels, onDiscover }: Props = $props();

  let collectiveOpen = $state(true);
  let channelsOpen = $state(true);
  let communitiesOpen = $state(true);
</script>

<section class="rail-panel">
  <details class="rail-section" bind:open={collectiveOpen}>
    <summary class="rail-section-summary">
      <span class="summary-label">
        <span class="chevron" aria-hidden="true"></span>
        <span>Collective</span>
      </span>
    </summary>
    <div class="stack-links">
      <RailLinkRow
        href={resolve('/platform')}
        icon="platform"
        label="Platform"
        onclick={closePanels}
      />
      <RailLinkRow
        href={resolve('/platform?board=1')}
        icon="shield"
        label="Moderators"
        onclick={closePanels}
      />
    </div>
  </details>
</section>

<section class="rail-panel">
  <details class="rail-section" bind:open={channelsOpen}>
    <summary class="rail-section-summary">
      <span class="summary-label">
        <span class="chevron" aria-hidden="true"></span>
        <span>Channels</span>
      </span>
    </summary>
    <div class="stack-links">
      {#each channels as link (link.href)}
        <RailLinkRow
          href={link.href}
          icon="channel"
          imageUrl={link.imageUrl ?? null}
          label={link.label}
          onclick={closePanels}
        />
      {:else}
        <p class="empty-hint">Join channels to see them here.</p>
      {/each}
    </div>
  </details>
</section>

<section class="rail-panel">
  <details class="rail-section" bind:open={communitiesOpen}>
    <summary class="rail-section-summary">
      <span class="summary-label">
        <span class="chevron" aria-hidden="true"></span>
        <span>Communities</span>
      </span>
    </summary>
    <div class="stack-links">
      {#if invites > 0}
        <!-- Invites to private communities and events wait here until answered. -->
        <RailLinkRow
          href={resolve('/invites')}
          icon="community"
          label="Invites"
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
          onclick={closePanels}
        />
      {:else}
        <p class="empty-hint">Join communities to see them here.</p>
      {/each}
    </div>
  </details>
</section>

<section class="rail-panel">
  <div class="action-stack">
    <button class="rail-action discover-button" type="button" onclick={onDiscover}>
      <ToolbarIcon name="plus" />
      <span>Discover</span>
    </button>

    <a class="rail-action feedback-button" href={resolve('/feedback')} onclick={closePanels}>
      <ToolbarIcon name="pencil" />
      <span>Feedback</span>
    </a>
  </div>
</section>

<style>
  .invite-count {
    min-width: 20px;
    padding: 1px 6px;
    border-radius: 999px;
    background: var(--brand);
    color: var(--on-brand, #fff);
    font-size: 12px;
    font-weight: 800;
    text-align: center;
  }

  .rail-panel {
    padding: 0 0 12px;
    border-bottom: 1px solid var(--panel-border);
  }

  .rail-section {
    display: grid;
  }

  .rail-section[open] > .rail-section-summary {
    margin-bottom: 8px;
  }

  .rail-section-summary {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    min-height: 28px;
    cursor: pointer;
    list-style: none;
    font-size: 14px;
    font-weight: 700;
    color: var(--text-main);
  }

  .rail-section-summary::-webkit-details-marker {
    display: none;
  }

  .summary-label {
    display: inline-flex;
    align-items: center;
    gap: 8px;
    min-width: 0;
  }

  .chevron {
    width: 0;
    height: 0;
    border-top: 5px solid transparent;
    border-bottom: 5px solid transparent;
    border-left: 6px solid currentColor;
    transition: transform 0.16s ease;
    flex-shrink: 0;
  }

  details[open] > .rail-section-summary .chevron {
    transform: rotate(90deg);
  }

  .action-stack {
    display: grid;
    gap: 8px;
    width: 100%;
    padding-top: 2px;
  }

  .rail-action {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    gap: 8px;
    width: 100%;
    min-height: 38px;
    padding: 0 14px;
    border-radius: var(--radius-sm);
    font-size: 13px;
    font-weight: 800;
    text-decoration: none;
    cursor: pointer;
  }

  .discover-button {
    border: 1px solid color-mix(in srgb, var(--brand) 74%, transparent);
    background: var(--brand);
    color: var(--on-brand);
  }

  .discover-button:hover {
    background: color-mix(in srgb, var(--brand) 86%, black);
  }

  .feedback-button {
    border: 1px solid color-mix(in srgb, var(--panel-border) 88%, transparent);
    background: color-mix(in srgb, var(--panel-soft) 90%, transparent);
    color: var(--text-main);
  }

  .feedback-button:hover {
    background: color-mix(in srgb, var(--brand) 10%, var(--panel-soft));
    border-color: color-mix(in srgb, var(--brand) 40%, var(--panel-border));
  }

  .stack-links {
    display: grid;
    gap: 2px;
  }

  .empty-hint {
    margin: 0;
    padding: 4px 10px;
    color: var(--text-soft);
    font-size: 12px;
    line-height: 1.45;
  }
</style>

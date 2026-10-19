<script lang="ts">
  import ToolbarIcon from '#lib/components/icons/ToolbarIcon.svelte';
  import type { DetailTabId } from '#lib/domain/projects.ts';

  import '#lib/components/styles/detail.css';

  /** web's DetailTopTabs. `available` lists the tabs built so far (M3–M5 add the rest). */
  interface Props {
    active: DetailTabId;
    available: DetailTabId[];
    ariaLabel: string;
    onSelect: (tab: DetailTabId) => void;
  }

  let { active, available, ariaLabel, onSelect }: Props = $props();

  const all: { id: DetailTabId; label: string; short?: string; icon: string }[] = [
    { id: 'context', label: 'Details', icon: 'list' },
    { id: 'participation', label: 'Participation', short: 'Take part', icon: 'people' },
    { id: 'chat', label: 'Chat', icon: 'message' },
    { id: 'threads', label: 'Threads', icon: 'thread' },
    { id: 'links', label: 'Links', icon: 'link' },
    { id: 'history', label: 'History', icon: 'clock' },
  ];
  const tabs = $derived(all.filter((t) => available.includes(t.id)));
</script>

<div
  class="top-tab-row"
  class:chat-immersive={active === 'chat'}
  role="tablist"
  aria-label={ariaLabel}
>
  {#each tabs as tab (tab.id)}
    <button
      class:active-tab={active === tab.id}
      class:participation-tab={tab.id === 'participation'}
      class="top-tab detail-surface-tab"
      role="tab"
      type="button"
      aria-label={tab.label}
      title={tab.label}
      aria-selected={active === tab.id}
      onclick={() => onSelect(tab.id)}
    >
      <span class="tab-icon" aria-hidden="true"><ToolbarIcon name={tab.icon} /></span>
      <span class="tab-label">{tab.label}</span>
      {#if tab.short}<span class="tab-label-short">{tab.short}</span>{/if}
    </button>
  {/each}
</div>

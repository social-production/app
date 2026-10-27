<script lang="ts">
  import { goto } from '$app/navigation';
  import { resolve } from '$app/paths';
  import { page } from '$app/state';

  import RailCleared from '#lib/components/rail/RailCleared.svelte';
  import RailHistory from '#lib/components/rail/RailHistory.svelte';
  import RailRow from '#lib/components/rail/RailRow.svelte';
  import RailSection from '#lib/components/rail/RailSection.svelte';
  import { railTime, type RailItem } from '#lib/domain/activityRail.ts';
  import { projectPageOf } from '#lib/services/notifications.ts';
  import {
    activityRail,
    dismissRailItem,
    markRailItemSeen,
    restoreAllRailItems,
    restoreRailItem,
  } from '#lib/stores/activityRail.svelte.ts';

  import '#lib/components/styles/rail.css';
  import { isEvent } from '#lib/domain/projects.ts';

  /**
   * web's RightRailPanel: Project Activity & Events, Help Requests, Requests, Votes to cast,
   * History and Cleared. Help and service requests stay empty until they exist (M5c).
   */
  let { onClose }: { onClose: () => void } = $props();

  const visible = $derived(
    activityRail.items.filter((i) => !activityRail.dismissed.includes(i.id)),
  );
  const cleared = $derived(activityRail.items.filter((i) => activityRail.dismissed.includes(i.id)));
  const activities = $derived(visible.filter((i) => i.kind === 'project'));
  const votes = $derived(visible.filter((i) => i.kind === 'vote'));
  const unseen = (item: RailItem): boolean => !activityRail.seen.includes(item.id);

  /** web's handleOpenItem; a link into the page already open reloads it, so it opens there. */
  async function openItem(item: RailItem): Promise<void> {
    if (item.kind !== 'vote') markRailItemSeen(item.id);
    onClose();
    const id = await projectPageOf(item.seed);
    if (!id) return;
    const path = isEvent(item.projectMode)
      ? resolve('/events/[id]', { id })
      : resolve('/projects/[id]', { id });
    if (page.url.pathname === path) {
      window.location.assign(`${path}${item.query}`);
      return;
    }
    // eslint-disable-next-line svelte/no-navigation-without-resolve -- the path is resolved above
    await goto(`${path}${item.query}`);
  }
</script>

<section class="rail-panel">
  <RailSection
    title="Project Activity & Events"
    count={activities.length}
    hot={activities.some(unseen)}
    emptyTitle="No activity yet"
    emptyText="Nothing scheduled in your memberships right now."
  >
    {#each activities as item (item.id)}
      <RailRow
        {item}
        unseen={unseen(item)}
        rowClass=""
        dismissLabel="Dismiss activity card"
        time={railTime(item.createdAt)}
        onOpen={() => openItem(item)}
        onDismiss={() => dismissRailItem(item.id)}
      />
    {/each}
  </RailSection>

  <RailSection
    title="Help Requests"
    count={0}
    hot={false}
    extraClass="rail-section-help-requests"
    emptyTitle="No help requests yet"
    emptyText="Open help in your memberships will show here."
  />

  <RailSection
    title="Requests"
    count={0}
    hot={false}
    extraClass="rail-section-requests"
    emptyTitle="No open requests"
    emptyText="Service requests you can review will appear here and open the matching project card."
  />

  <RailSection
    title="Votes to cast"
    count={votes.length}
    hot={votes.length > 0}
    extraClass="rail-section-votes"
    emptyTitle="No votes waiting"
    emptyText="Open decisions you still need to cast appear here. Votes you already cast stay in History until they close."
  >
    {#each votes as item (item.id)}
      <RailRow
        {item}
        unseen={false}
        rowClass="vote-row"
        dismissLabel="Dismiss vote card"
        time={railTime(item.createdAt)}
        onOpen={() => openItem(item)}
        onDismiss={() => dismissRailItem(item.id)}
      >
        {#snippet actions()}
          <div class="vote-row-actions">
            <button
              class="vote-action-button assess-button"
              type="button"
              onclick={() => openItem(item)}
            >
              Assess
            </button>
          </div>
        {/snippet}
      </RailRow>
    {/each}
  </RailSection>

  {#if activityRail.history.length > 0}
    <RailHistory items={activityRail.history} onOpen={openItem} />
  {/if}

  {#if cleared.length > 0}
    <RailCleared
      items={cleared}
      onOpen={openItem}
      onRestore={restoreRailItem}
      onRestoreAll={restoreAllRailItems}
    />
  {/if}
</section>

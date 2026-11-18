<script lang="ts">
  import { t } from '#lib/i18n/index.ts';
  import { goto } from '$app/navigation';
  import { page } from '$app/state';

  import EmptyState from '#lib/components/empty/EmptyState.svelte';
  import RailCleared from '#lib/components/rail/RailCleared.svelte';
  import RailHistory from '#lib/components/rail/RailHistory.svelte';
  import RailRow from '#lib/components/rail/RailRow.svelte';
  import { railTime, type RailItem } from '#lib/domain/activityRail.ts';
  import { isHelpKind } from '#lib/domain/helpRail.ts';
  import { projectPageOf } from '#lib/services/notifications.ts';
  import {
    activityRail,
    dismissRailItem,
    markRailItemSeen,
    restoreAllRailItems,
    restoreRailItem,
  } from '#lib/stores/activityRail.svelte.ts';

  import '#lib/components/styles/rail.css';
  import { spaceHref } from '#lib/components/cards/spaceHref.ts';

  /**
   * Your activity (Screen Review §1, owner 2026-11-18): three tabs with counts instead of web's
   * four disclosure lists. Coming up is what you're in (web's Project Activity & Events); Votes
   * is web's Votes to cast; Requests merges help requests (yours, your sign-ups, open ones in
   * your channels and communities) and service requests you can answer (ADR 0076). History and
   * Cleared stay below.
   */
  let { onClose }: { onClose: () => void } = $props();

  const visible = $derived(
    activityRail.items.filter((i) => !activityRail.dismissed.includes(i.id)),
  );
  const cleared = $derived(activityRail.items.filter((i) => activityRail.dismissed.includes(i.id)));
  const activities = $derived(visible.filter((i) => i.kind === 'project'));
  const votes = $derived(visible.filter((i) => i.kind === 'vote'));
  const requests = $derived(visible.filter((i) => i.kind === 'request'));
  const help = $derived(visible.filter((i) => isHelpKind(i.kind)));
  const unseen = (item: RailItem): boolean => !activityRail.seen.includes(item.id);
  type Tab = 'coming' | 'votes' | 'requests';
  let tab = $state<Tab>('coming');
  const tabs = $derived([
    {
      id: 'coming' as Tab,
      label: 'Coming up',
      count: activities.length,
      hot: activities.some(unseen),
    },
    { id: 'votes' as Tab, label: 'Votes', count: votes.length, hot: votes.length > 0 },
    {
      id: 'requests' as Tab,
      label: 'Requests',
      count: help.length + requests.length,
      hot: help.some(unseen) || requests.some(unseen),
    },
  ]);

  /** web's handleOpenItem; a link into the page already open reloads it, so it opens there. */
  async function openItem(item: RailItem): Promise<void> {
    if (item.kind !== 'vote') markRailItemSeen(item.id);
    onClose();
    const id = item.projectId ?? (await projectPageOf(item.seed));
    if (!id) return;
    const path = spaceHref(item.projectMode, id);
    if (page.url.pathname === path) {
      window.location.assign(`${path}${item.query}`);
      return;
    }
    // eslint-disable-next-line svelte/no-navigation-without-resolve -- the path is resolved above
    await goto(`${path}${item.query}`);
  }
</script>

<section class="rail-panel rail-tabs-panel">
  <h2 class="rail-title">{t('Your activity')}</h2>
  <div class="rail-tabs" role="tablist" aria-label={t('Your activity')}>
    {#each tabs as t (t.id)}
      <button
        type="button"
        role="tab"
        aria-selected={tab === t.id}
        class:on={tab === t.id}
        onclick={() => (tab = t.id)}
        >{t.label}{#if t.count > 0}<span class="rail-tab-count" class:hot={t.hot}>{t.count}</span
          >{/if}</button
      >
    {/each}
  </div>

  {#if tab === 'coming'}
    <div class="rail-tab-panel" role="tabpanel">
      {#each activities as item (item.id)}
        <RailRow
          {item}
          unseen={unseen(item)}
          rowClass=""
          dismissLabel={t('Dismiss activity card')}
          time={railTime(item.createdAt)}
          onOpen={() => openItem(item)}
          onDismiss={() => dismissRailItem(item.id)}
        />
      {:else}
        <EmptyState
          icon="calendar"
          title={t('No activity yet')}
          text={t('Activities and events you join show up here.')}
        />
      {/each}
    </div>
  {:else if tab === 'votes'}
    <div class="rail-tab-panel rail-section-votes" role="tabpanel">
      {#each votes as item (item.id)}
        <RailRow
          {item}
          unseen={false}
          rowClass="vote-row"
          dismissLabel={t('Dismiss vote card')}
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
                {item.voteKindLabel === 'pull_request_merge' ? t('Record merge') : t('Assess')}
              </button>
            </div>
          {/snippet}
        </RailRow>
      {:else}
        <EmptyState
          icon="check"
          title={t('No votes waiting')}
          text={t(
            'Decisions that need your vote show up here. Once you vote, they move to History.',
          )}
        />
      {/each}
    </div>
  {:else}
    <div class="rail-tab-panel" role="tabpanel">
      <div class="rail-section-help-requests">
        {#each help as item (item.id)}
          <RailRow
            {item}
            unseen={unseen(item)}
            rowClass="help-request-row"
            dismissLabel={t('Dismiss help request card')}
            time={railTime(item.createdAt)}
            onOpen={() => openItem(item)}
            onDismiss={() => dismissRailItem(item.id)}
          />
        {/each}
      </div>
      <div class="rail-section-requests">
        {#each requests as item (item.id)}
          <RailRow
            {item}
            unseen={unseen(item)}
            rowClass=""
            dismissLabel={t('Dismiss request card')}
            time={railTime(item.createdAt)}
            onOpen={() => openItem(item)}
            onDismiss={() => dismissRailItem(item.id)}
          />
        {/each}
      </div>
      {#if help.length + requests.length === 0}
        <EmptyState
          icon="message"
          title={t('No requests to answer')}
          text={t('Help and service requests in your projects and channels show up here.')}
        />
      {/if}
    </div>
  {/if}

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

<script lang="ts">
  import { untrack } from 'svelte';

  import { goto } from '$app/navigation';
  import { resolve } from '$app/paths';
  import NotificationCard from '#lib/components/cards/NotificationCard.svelte';
  import type { NotificationItem } from '#lib/domain/notifications.ts';
  import { explain } from '#lib/services/errors.ts';
  import {
    markAllNotificationsRead,
    markNotificationRead,
    myNotifications,
    projectPageOf,
  } from '#lib/services/notifications.ts';
  import { namesOf } from '#lib/services/people.ts';
  import { notifications, showNotifications } from '#lib/stores/notifications.svelte.ts';

  import '#lib/components/styles/messages.css';
  import '#lib/components/styles/notifications.css';

  /** web's NotificationsPage: the list, newest first, with "Mark all read". */
  let names = $state<Record<string, string>>({});
  let opening = $state('');

  let error = $state('');

  $effect(() => {
    untrack(() => void refresh());
  });

  async function refresh(): Promise<void> {
    try {
      showNotifications((await myNotifications()).items);
      error = '';
    } catch (e) {
      error = explain(e);
    }
  }

  /** web's "Mark read" (also when a card is opened). */
  async function read(item: NotificationItem): Promise<void> {
    if (!item.isUnread) return;
    await markNotificationRead(item, notifications.items);
    showNotifications(
      notifications.items.map((n) => (n.id === item.id ? { ...n, isUnread: false } : n)),
    );
  }

  async function readAll(): Promise<void> {
    await markAllNotificationsRead(notifications.items);
    showNotifications(notifications.items.map((n) => ({ ...n, isUnread: false })));
  }

  $effect(() => {
    const missing = [...new Set(notifications.items.map((n) => n.actor))].filter(
      (a) => !(a in names),
    );
    if (missing.length === 0) return;
    void namesOf(missing).then((found) => (names = { ...names, ...found }));
  });

  /** web's activateNotification: read it, then open the project where it happened. */
  async function activate(item: NotificationItem): Promise<void> {
    if (opening) return;
    opening = item.id;
    try {
      await read(item);
      const id = await projectPageOf(item.seed);
      if (!id) return;
      const page = resolve('/projects/[id]', { id });
      const t = item.target;
      const query = !t
        ? ''
        : 'tab' in t
          ? `?tab=chat&comment=${encodeURIComponent(t.comment)}`
          : `?open=vote&voteKind=phase_change&voteTarget=${encodeURIComponent(t.voteTarget)}`;
      // eslint-disable-next-line svelte/no-navigation-without-resolve -- the path is resolved above
      await goto(`${page}${query}`);
    } finally {
      opening = '';
    }
  }
</script>

<section class="page notifications-page">
  <header class="page-header">
    <div class="copy">
      <h1>Notifications</h1>
      <p>Project activity and event invites from your network.</p>
    </div>
    <div class="actions">
      <button class="secondary-button" type="button" onclick={readAll}>Mark all read</button>
    </div>
  </header>

  {#if error}
    <p class="error-note">{error}</p>
  {/if}

  <div class="stack">
    {#each notifications.items as item (item.id)}
      <NotificationCard
        {item}
        actorName={names[item.actor] ?? 'Someone'}
        onRead={() => read(item)}
        onActivate={() => activate(item)}
      />
    {/each}
  </div>
</section>

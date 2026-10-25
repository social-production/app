<script lang="ts">
  import type { NotificationCategory } from '#lib/domain/notifications.ts';
  import { notificationsOff, setNotificationCategory } from '#lib/stores/notifications.svelte.ts';

  /** web's Notifications settings for the kinds the app has; on this device. */
  const options: { id: NotificationCategory; title: string; description: string }[] = [
    {
      id: 'comments',
      title: 'Comments',
      description: 'Replies and mentions on your posts and activity.',
    },
    {
      id: 'votes_needed',
      title: 'Votes needed',
      description: 'Phase votes and approvals that need your input.',
    },
    {
      id: 'phase_done',
      title: 'Phase finished',
      description: 'When a project or event phase is completed.',
    },
  ];
  const on = (id: NotificationCategory): boolean => !notificationsOff.categories.includes(id);
</script>

<section class="settings-section" id="settings-notifications">
  <h2>Notifications</h2>
  <p class="section-intro">Choose which activity shows up in your notifications.</p>
  <div class="card stack flush">
    {#each options as option (option.id)}
      {@const enabled = on(option.id)}
      <div class="setting-item">
        <div>
          <strong>{option.title}</strong>
          <p>{option.description}</p>
        </div>
        <button
          aria-checked={enabled}
          aria-label={option.title}
          class="switch"
          class:on={enabled}
          role="switch"
          type="button"
          onclick={() => setNotificationCategory(option.id, !enabled)}
        >
          <span class="switch-thumb"></span>
          <span class="sr-only">{enabled ? 'On' : 'Off'}</span>
        </button>
      </div>
    {/each}
  </div>
</section>

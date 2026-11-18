<script lang="ts">
  import { t } from '#lib/i18n/index.ts';
  import SettingSwitch from '#lib/components/settings/SettingSwitch.svelte';
  import type { NotificationCategory } from '#lib/domain/notificationCategories.ts';
  import { notificationsOff, setNotificationCategory } from '#lib/stores/notifications.svelte.ts';
  import { pushChoicesChanged } from '#lib/stores/push.svelte.ts';

  import PushSettings from './PushSettings.svelte';

  /** web's Notifications settings for the kinds the app has; on this device. */
  const options: { id: NotificationCategory; title: string; description: string }[] = [
    {
      id: 'follows',
      title: t('Follows'),
      description: t('Follow requests, new followers, and accepted requests.'),
    },
    {
      id: 'comments',
      title: t('Comments'),
      description: t('Replies and mentions on your posts and activity.'),
    },
    {
      id: 'shares_invites',
      title: t('Shares and invites'),
      description: t('Shared projects or events, and invites to join.'),
    },
    {
      id: 'roles',
      title: t('Roles'),
      description: t('Role sign-ups and suggested roles on your work.'),
    },
    {
      id: 'votes_needed',
      title: t('Votes needed'),
      description: t('Phase votes and approvals that need your input.'),
    },
    {
      id: 'phase_done',
      title: t('Phase finished'),
      description: t('When a project or event phase is completed.'),
    },
    {
      id: 'plan_leading',
      title: t('Plan became leading'),
      description: t('When a plan becomes the leading option. Off by default.'),
    },
  ];
  const on = (id: NotificationCategory): boolean => !notificationsOff.categories.includes(id);
</script>

<section class="settings-section" id="settings-notifications">
  <h2>{t('Notifications')}</h2>
  <p class="section-intro">{t('Choose which activity shows up in your notifications.')}</p>
  <div class="card stack flush">
    {#each options as option (option.id)}
      {@const enabled = on(option.id)}
      <SettingSwitch
        label={option.title}
        note={option.description}
        checked={enabled}
        onToggle={() => {
          setNotificationCategory(option.id, !enabled);
          void pushChoicesChanged();
        }}
      />
    {/each}
  </div>
  <PushSettings />
</section>

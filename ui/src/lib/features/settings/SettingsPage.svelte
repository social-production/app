<script lang="ts">
  import { t } from '#lib/i18n/index.ts';
  import { inNativeApp } from '#lib/services/network.ts';
  import { connection } from '#lib/stores/connection.svelte.ts';

  import AccountSettings from './AccountSettings.svelte';
  import AppearanceSettings from './AppearanceSettings.svelte';
  import DeviceSettings from './DeviceSettings.svelte';
  import FollowSettings from './FollowSettings.svelte';
  import NetworkSettings from './NetworkSettings.svelte';
  import NotificationSettings from './NotificationSettings.svelte';
  import ProfileSettings from './ProfileSettings.svelte';
  import RegionalSettings from './RegionalSettings.svelte';

  import '#lib/components/styles/settings.css';

  /** web's settings layout: a section nav beside cards (Profile, Appearance, Regional, Notifications, Devices, Follow requests, Privacy, Account). */
  let requestCount = $state(0);
  const native = inNativeApp();
  const me = $derived(connection.current.status === 'connected' ? connection.current.agent : '');
</script>

<div class="settings-scope">
  <section class="settings-layout">
    <nav class="settings-nav" aria-label={t('Settings sections')}>
      <p class="settings-nav-label">{t('Settings')}</p>
      <a class="nav-item" href="#settings-profile">{t('Profile')}</a>
      <a class="nav-item" href="#settings-appearance">{t('Appearance')}</a>
      <a class="nav-item" href="#settings-regional">{t('Regional')}</a>
      <a class="nav-item" href="#settings-notifications">{t('Notifications')}</a>
      <a class="nav-item" href="#settings-devices">{t('Devices')}</a>
      {#if native}
        <a class="nav-item" href="#settings-network">{t('Network')}</a>
      {/if}
      {#if requestCount > 0}
        <a class="nav-item" href="#settings-follow-requests">{t('Follow requests')}</a>
      {/if}
      <a class="nav-item" href="#settings-privacy">{t('Privacy')}</a>
      <a class="nav-item" href="#settings-account">{t('Account')}</a>
    </nav>

    <section class="settings-page">
      <header class="page-header">
        <h1>{t('Settings')}</h1>
        <p>{t('Your profile, appearance, notifications, devices, privacy and sign-in.')}</p>
      </header>

      {#if me}
        <ProfileSettings {me} />
        <AppearanceSettings />
        <RegionalSettings {me} />
        <NotificationSettings />
        <DeviceSettings {me} />
        <NetworkSettings />
        <FollowSettings {me} onRequests={(n) => (requestCount = n)} />
        <AccountSettings {me} />
      {/if}
    </section>
  </section>
</div>

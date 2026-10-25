<script lang="ts">
  import { connection } from '#lib/stores/connection.svelte.ts';

  import AccountSettings from './AccountSettings.svelte';
  import AppearanceSettings from './AppearanceSettings.svelte';
  import DeviceSettings from './DeviceSettings.svelte';
  import FollowSettings from './FollowSettings.svelte';
  import NotificationSettings from './NotificationSettings.svelte';
  import ProfileSettings from './ProfileSettings.svelte';

  import '#lib/components/styles/settings.css';

  /** web's settings layout: a section nav beside cards (Profile, Appearance, Notifications, Devices, Follow requests, Privacy, Account). */
  let requestCount = $state(0);
  const me = $derived(connection.current.status === 'connected' ? connection.current.agent : '');
</script>

<div class="settings-scope">
  <section class="settings-layout">
    <nav class="settings-nav" aria-label="Settings sections">
      <p class="settings-nav-label">Settings</p>
      <a class="nav-item" href="#settings-profile">Profile</a>
      <a class="nav-item" href="#settings-appearance">Appearance</a>
      <a class="nav-item" href="#settings-notifications">Notifications</a>
      <a class="nav-item" href="#settings-devices">Devices</a>
      {#if requestCount > 0}
        <a class="nav-item" href="#settings-follow-requests">Follow requests</a>
      {/if}
      <a class="nav-item" href="#settings-privacy">Privacy</a>
      <a class="nav-item" href="#settings-account">Account</a>
    </nav>

    <section class="settings-page">
      <header class="page-header">
        <h1>Settings</h1>
        <p>Your profile, appearance, notifications, devices, privacy and sign-in.</p>
      </header>

      {#if me}
        <ProfileSettings {me} />
        <AppearanceSettings />
        <NotificationSettings />
        <DeviceSettings {me} />
        <FollowSettings {me} onRequests={(n) => (requestCount = n)} />
        <AccountSettings {me} />
      {/if}
    </section>
  </section>
</div>

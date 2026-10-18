<script lang="ts">
  import { connection } from '#lib/stores/connection.svelte.ts';

  import AccountSettings from './AccountSettings.svelte';
  import DeviceSettings from './DeviceSettings.svelte';
  import ProfileSettings from './ProfileSettings.svelte';

  import '#lib/components/styles/settings.css';

  /** web's settings layout: a section nav beside cards (Profile, Devices, Account). */
  const me = $derived(connection.current.status === 'connected' ? connection.current.agent : '');
</script>

<div class="settings-scope">
  <section class="settings-layout">
    <nav class="settings-nav" aria-label="Settings sections">
      <p class="settings-nav-label">Settings</p>
      <a class="nav-item" href="#settings-profile">Profile</a>
      <a class="nav-item" href="#settings-devices">Devices</a>
      <a class="nav-item" href="#settings-account">Account</a>
    </nav>

    <section class="settings-page">
      <header class="page-header">
        <h1>Settings</h1>
        <p>Your profile, devices and sign-in.</p>
      </header>

      {#if me}
        <ProfileSettings {me} />
        <DeviceSettings {me} />
        <AccountSettings {me} />
      {/if}
    </section>
  </section>
</div>

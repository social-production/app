<script lang="ts">
  import AccountGate from '#lib/features/account/AccountGate.svelte';
  import SettingsPage from '#lib/features/settings/SettingsPage.svelte';
</script>

<AccountGate>
  <SettingsPage />
</AccountGate>

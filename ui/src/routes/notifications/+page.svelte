<script lang="ts">
  import AccountGate from '#lib/features/account/AccountGate.svelte';
  import NotificationsPage from '#lib/features/notifications/NotificationsPage.svelte';
</script>

<AccountGate>
  <NotificationsPage />
</AccountGate>

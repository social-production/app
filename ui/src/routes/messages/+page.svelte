<script lang="ts">
  import AccountGate from '#lib/features/account/AccountGate.svelte';
  import MessagesPage from '#lib/features/messages/MessagesPage.svelte';
</script>

<AccountGate>
  <MessagesPage />
</AccountGate>

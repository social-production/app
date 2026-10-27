<script lang="ts">
  import AccountGate from '#lib/features/account/AccountGate.svelte';
  import InvitesPage from '#lib/features/invites/InvitesPage.svelte';
</script>

<AccountGate>
  <InvitesPage />
</AccountGate>

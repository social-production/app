<script lang="ts">
  import AccountGate from '#lib/features/account/AccountGate.svelte';
  import CreateHelpRequest from '#lib/features/help-requests/CreateHelpRequest.svelte';
</script>

<AccountGate>
  <CreateHelpRequest />
</AccountGate>

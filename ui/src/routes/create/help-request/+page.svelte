<script lang="ts">
  import AccountGate from '#lib/features/account/AccountGate.svelte';
  import CreateHelpRequest from '#lib/features/projects/help/CreateHelpRequest.svelte';
</script>

<AccountGate>
  <CreateHelpRequest />
</AccountGate>

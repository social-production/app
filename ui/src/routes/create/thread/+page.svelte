<script lang="ts">
  import AccountGate from '#lib/features/account/AccountGate.svelte';
  import CreateThread from '#lib/features/projects/discussions/CreateThread.svelte';
</script>

<AccountGate>
  <CreateThread />
</AccountGate>

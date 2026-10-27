<script lang="ts">
  import AccountGate from '#lib/features/account/AccountGate.svelte';
  import CreateEvent from '#lib/features/projects/CreateEvent.svelte';
</script>

<AccountGate>
  <CreateEvent />
</AccountGate>

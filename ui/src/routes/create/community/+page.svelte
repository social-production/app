<script lang="ts">
  import AccountGate from '#lib/features/account/AccountGate.svelte';
  import CreateScope from '#lib/features/projects/scopes/CreateScope.svelte';
</script>

<AccountGate>
  <CreateScope kind="community" />
</AccountGate>

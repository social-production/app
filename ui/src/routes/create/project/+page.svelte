<script lang="ts">
  import AccountGate from '#lib/features/account/AccountGate.svelte';
  import CreateProject from '#lib/features/projects/CreateProject.svelte';
</script>

<AccountGate>
  <CreateProject />
</AccountGate>

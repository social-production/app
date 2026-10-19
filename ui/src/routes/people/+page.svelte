<script lang="ts">
  import AccountGate from '#lib/features/account/AccountGate.svelte';
  import PeoplePage from '#lib/features/people/PeoplePage.svelte';
</script>

<AccountGate>
  <PeoplePage />
</AccountGate>

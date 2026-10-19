<script lang="ts">
  import AccountGate from '#lib/features/account/AccountGate.svelte';
  import HealthPanel from '#lib/features/health/HealthPanel.svelte';
</script>

<AccountGate>
  <HealthPanel />
</AccountGate>

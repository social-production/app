<script lang="ts">
  import type { AccountStep } from '#lib/services/account.ts';

  let { current }: { current: AccountStep } = $props();

  const labels: Record<AccountStep, string> = {
    checking: 'Proving you are not a bot (a few seconds)…',
    passkey: 'Confirm with your passkey when your device asks…',
    connecting: 'Setting up your node…',
  };
</script>

<p class="active helper-text" role="status">{labels[current]}</p>

<style>
  .active {
    font-weight: 600;
  }
</style>

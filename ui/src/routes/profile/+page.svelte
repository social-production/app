<script lang="ts">
  import AccountGate from '#lib/features/account/AccountGate.svelte';
  import OwnProfile from '#lib/features/people/OwnProfile.svelte';
</script>

<AccountGate>
  <OwnProfile />
</AccountGate>

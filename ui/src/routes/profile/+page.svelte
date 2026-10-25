<script lang="ts">
  import AccountGate from '#lib/features/account/AccountGate.svelte';
  import OwnProfile from '#lib/features/people/OwnProfile.svelte';
  import ProfileFeed from '#lib/features/projects/ProfileFeed.svelte';
</script>

<AccountGate>
  <OwnProfile>
    <ProfileFeed />
  </OwnProfile>
</AccountGate>

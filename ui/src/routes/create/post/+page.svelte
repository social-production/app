<script lang="ts">
  import AccountGate from '#lib/features/account/AccountGate.svelte';
  import CreatePost from '#lib/features/projects/discussions/CreatePost.svelte';
</script>

<AccountGate>
  <CreatePost />
</AccountGate>

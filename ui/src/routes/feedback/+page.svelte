<script lang="ts">
  import AccountGate from '#lib/features/account/AccountGate.svelte';
  import FeedbackPage from '#lib/features/feedback/FeedbackPage.svelte';
</script>

<AccountGate guests>
  <FeedbackPage />
</AccountGate>

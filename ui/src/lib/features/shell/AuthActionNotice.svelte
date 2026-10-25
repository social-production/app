<script lang="ts">
  import { resolve } from '$app/paths';
  import { page } from '$app/state';

  import { authNotice, dismissAuthNotice } from '#lib/stores/authNotice.svelte.ts';

  import '#lib/components/styles/notice.css';

  /** web's AuthActionNotice: shown when a guest tries something that needs an account. */
  let lastPath = '';

  $effect(() => {
    const path = page.url.pathname;
    if (lastPath && path !== lastPath) dismissAuthNotice();
    lastPath = path;
  });
</script>

{#if authNotice.message}
  <div class="auth-action-notice" role="status">
    <span>{authNotice.message}</span>
    <a href={resolve('/onboarding')}>Sign in</a>
    <button class="dismiss" type="button" aria-label="Dismiss" onclick={dismissAuthNotice}>×</button
    >
  </div>
{/if}

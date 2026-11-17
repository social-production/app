<script lang="ts">
  import { goto } from '$app/navigation';
  import { resolve } from '$app/paths';
  import { page } from '$app/state';

  import { authNotice, dismissAuthNotice } from '#lib/stores/authNotice.svelte.ts';

  import '#lib/components/styles/notice.css';

  /** web's AuthActionNotice, now only for why an account was signed out; a guest who tries
   * something that needs an account goes straight to the sign-up page. */
  let lastPath = '';

  // A guest who tries something that needs an account goes to the sign-up page.
  let asked = 0;
  $effect(() => {
    if (authNotice.signIn === asked) return;
    asked = authNotice.signIn;
    void goto(resolve('/onboarding'));
  });

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

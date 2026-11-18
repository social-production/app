<script lang="ts">
  import { t } from '#lib/i18n/index.ts';
  import type { Snippet } from 'svelte';

  import { connect, signOut } from '#lib/services/account.ts';
  import { connection } from '#lib/stores/connection.svelte.ts';

  import { goto } from '$app/navigation';
  import { resolve } from '$app/paths';
  import { page } from '$app/state';

  import OpeningScreen from '#lib/components/loading/OpeningScreen.svelte';

  import '#lib/components/styles/onboarding.css';

  /** `guests`: signed-out people see the page too (web's public pages; reads go through the
   * gateway, holo/guest.ts). */
  let { children, guests = false }: { children: Snippet; guests?: boolean } = $props();

  // The shell connects on every page; this only decides what an account-only page shows.
  // Signed out, it goes to the real sign-up page (owner, 2026-11-19: an inline one kept the
  // header and scrolled), which comes back here once you're in.
  const away = $derived(connection.current.status === 'signed-out' && !guests);
  $effect(() => {
    if (!away) return;
    const next = page.url.pathname + page.url.search;
    // eslint-disable-next-line svelte/no-navigation-without-resolve -- resolved, plus a query
    void goto(`${resolve('/onboarding')}?next=${encodeURIComponent(next)}`, {
      replaceState: true,
    });
  });
</script>

{#if connection.current.status === 'connected' || (guests && connection.current.status === 'signed-out')}
  {@render children()}
{:else if connection.current.status === 'offline'}
  <div class="onboarding">
    <section class="page">
      <section class="hero-card">
        <h1>{t("Can't reach the network")}</h1>
        <p class="status-note">{connection.current.reason}</p>
        <div class="button-row">
          <button class="button-primary" type="button" onclick={() => connect()}
            >{t('Try again')}</button
          >
          <button class="button-secondary" type="button" onclick={() => signOut()}
            >{t('Sign in again')}</button
          >
        </div>
      </section>
    </section>
  </div>
{:else}
  <OpeningScreen />
{/if}

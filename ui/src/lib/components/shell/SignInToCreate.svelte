<script lang="ts">
  import { t } from '#lib/i18n/index.ts';
  import OverlaySheet from '#lib/components/overlay/OverlaySheet.svelte';
  import '#lib/components/styles/ux/passkeysheet.css';

  import type { ResolvedPathname } from '$app/types';

  /** Signed out, the create + explains instead of jumping to sign-up (owner, 2026-11-23): a
   * short sheet in the passkey offer's style, with one way on. */
  interface Props {
    open: boolean;
    signInHref: ResolvedPathname;
    onClose: () => void;
  }

  let { open, signInHref, onClose }: Props = $props();
</script>

<!-- eslint-disable svelte/no-navigation-without-resolve -- the href arrives resolved -->
<OverlaySheet
  {open}
  title={t('Sign in to create')}
  labelledById="sign-in-to-create-title"
  rootClass="passkey-sheet"
  {onClose}
>
  {#snippet kicker()}
    <span class="passkey-badge" aria-hidden="true">
      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <path d="M12 5v14M5 12h14" />
      </svg>
    </span>
  {/snippet}
  <p class="passkey-lead">
    {t(
      'Projects, events, posts and requests come from members. Making an account takes a minute and needs no email.',
    )}
  </p>
  {#snippet footer()}
    <div class="sheet-actions">
      <a class="sheet-submit" href={signInHref} onclick={onClose}>{t('Sign up or log in')}</a>
    </div>
  {/snippet}
</OverlaySheet>

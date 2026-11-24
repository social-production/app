<script lang="ts">
  import { t } from '#lib/i18n/index.ts';
  import OverlaySheet from '#lib/components/overlay/OverlaySheet.svelte';
  import '#lib/components/styles/ux/passkeysheet.css';

  /**
   * Why a New member can't vote or start a vote (ADR 0116; owner, 2026-11-24: "a popup when
   * trying to change phase ... that full members are required").
   */
  interface Props {
    open: boolean;
    /** Members of this space with Full standing. */
    fullMembers: number;
    onClose: () => void;
  }

  let { open, fullMembers, onClose }: Props = $props();
</script>

<OverlaySheet
  {open}
  title={t('Only Full members vote')}
  labelledById="full-only-title"
  rootClass="passkey-sheet"
  {onClose}
>
  {#snippet kicker()}
    <span class="passkey-badge" aria-hidden="true">
      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8">
        <path d="M12 3l7 3v5c0 4.5-3 8.3-7 10-4-1.7-7-5.5-7-10V6l7-3z" />
        <path d="M9 12l2 2 4-4" />
      </svg>
    </span>
  {/snippet}
  <p class="passkey-lead">
    {t(
      "You're a New member, so you can't vote or start a vote yet. You get Full standing from two vouches by Full members, or after steady activity.",
    )}
  </p>
  <p class="passkey-lead">
    {fullMembers === 0
      ? t(
          'This has no Full members yet, so nothing here can go to a vote until one joins. Invite someone with Full standing.',
        )
      : fullMembers === 1
        ? t('1 Full member here decides its votes.')
        : t('{n} Full members here decide its votes.', { n: fullMembers })}
  </p>
  {#snippet footer()}
    <div class="sheet-actions">
      <button class="sheet-submit" type="button" onclick={onClose}>{t('OK')}</button>
    </div>
  {/snippet}
</OverlaySheet>

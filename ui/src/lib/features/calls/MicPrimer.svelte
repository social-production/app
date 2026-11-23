<script lang="ts">
  import { t } from '#lib/i18n/index.ts';
  import OverlaySheet from '#lib/components/overlay/OverlaySheet.svelte';
  import '#lib/components/styles/ux/passkeysheet.css';

  import { micAnswer, micAsk } from './micPrimer.svelte.ts';

  /** The microphone sheet before the first call (Owner Notes Mockups §8). */
</script>

<OverlaySheet
  open={micAsk.open}
  title={micAsk.blocked ? t('Your microphone is blocked') : t('Allow your microphone for calls')}
  labelledById="mic-primer-title"
  rootClass="passkey-sheet"
  onClose={() => micAnswer(false)}
>
  {#snippet kicker()}
    <span class="passkey-badge" aria-hidden="true">
      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8">
        <rect x="9" y="3" width="6" height="11" rx="3" />
        <path d="M5 11a7 7 0 0 0 14 0M12 18v3" />
      </svg>
    </span>
  {/snippet}
  <p class="passkey-lead">
    {micAsk.blocked
      ? t(
          "Allow the microphone for this site in your browser's settings (the icon by the address, or the site settings), then call again.",
        )
      : t('Your phone will ask next. Calls go straight between you and the other person.')}
  </p>
  {#snippet footer()}
    <div class="sheet-actions">
      <button class="sheet-submit" type="button" onclick={() => micAnswer(true)}
        >{micAsk.blocked ? t('OK') : t('Allow microphone')}</button
      >
    </div>
  {/snippet}
</OverlaySheet>

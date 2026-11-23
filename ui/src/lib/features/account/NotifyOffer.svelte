<script lang="ts">
  import { t } from '#lib/i18n/index.ts';
  import OverlaySheet from '#lib/components/overlay/OverlaySheet.svelte';
  import { devicePasskeysAvailable } from '#lib/services/account.ts';
  import { connection } from '#lib/stores/connection.svelte.ts';
  import { loadPush, push, setPushOn } from '#lib/stores/push.svelte.ts';
  import '#lib/components/styles/ux/passkeysheet.css';

  /** Push notifications offered once per device (Owner Notes Mockups §8), in the passkey
   * offer's style and after it: never both sheets at once. × or the scrim means Not now; the
   * switch stays in Settings. */
  const KEY = 'sp-push-offer';
  const PASSKEY_KEY = 'sp-passkey-offer';

  let visible = $state(false);

  const stored = (key: string): string | null => {
    try {
      return localStorage.getItem(key);
    } catch {
      return 'blocked'; // Storage refused: don't nag on every visit.
    }
  };

  $effect(() => {
    if (connection.current.status !== 'connected' || stored(KEY) !== null) return;
    if (typeof Notification === 'undefined' || Notification.permission !== 'default') return;
    if (navigator.webdriver) return; // Browser tests drive the page; the sheet would block them.
    void (async () => {
      await loadPush();
      if (push.where !== 'browser' || push.on) return;
      // The passkey offer goes first; this one waits for its answer.
      if (stored(PASSKEY_KEY) === null && (await devicePasskeysAvailable())) return;
      visible = true;
    })();
  });

  function close(answer: 'on' | 'dismissed'): void {
    visible = false;
    try {
      localStorage.setItem(KEY, answer);
    } catch {
      // The offer may come back next time.
    }
  }

  async function turnOn(): Promise<void> {
    await setPushOn(true);
    close('on');
  }
</script>

<OverlaySheet
  open={visible}
  title={t('Get told when someone replies')}
  labelledById="notify-offer-title"
  rootClass="passkey-sheet"
  onClose={() => close('dismissed')}
>
  {#snippet kicker()}
    <span class="passkey-badge" aria-hidden="true">
      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8">
        <path d="M6 16V11a6 6 0 0 1 12 0v5l2 2H4zM10 20a2 2 0 0 0 4 0" />
      </svg>
    </span>
  {/snippet}
  <p class="passkey-lead">
    {t('Replies, votes you are asked for, roles and calls. Choose which ones in Settings.')}
  </p>
  {#snippet footer()}
    <div class="sheet-actions">
      <button class="sheet-submit" type="button" onclick={turnOn} disabled={push.busy}
        >{push.busy ? t('Working…') : t('Turn on notifications')}</button
      >
    </div>
  {/snippet}
</OverlaySheet>

<script lang="ts">
  import { t } from '#lib/i18n/index.ts';
  import OverlaySheet from '#lib/components/overlay/OverlaySheet.svelte';
  import type { ReportReason } from '#lib/domain/moderation.ts';
  import '#lib/components/styles/report.css';

  /**
   * Reporting (Chats, Search and Reports mockups §4): why, as two cards with examples (the
   * two reasons a report records, ADR 0071), then what's wrong; once sent, the same sheet
   * says it's under review.
   */
  interface Props {
    open: boolean;
    itemLabel: string;
    pending: boolean;
    /** Why the last submit was refused. */
    error: string;
    /** The report went in: the sheet shows the confirmation until Done. */
    sent?: boolean;
    reason: ReportReason;
    description: string;
    onClose: () => void;
    onSubmit: () => void;
  }

  let {
    open,
    itemLabel,
    pending,
    error,
    sent = false,
    reason = $bindable(),
    description = $bindable(),
    onClose,
    onSubmit,
  }: Props = $props();

  const REASONS: { value: ReportReason; label: string; examples: string }[] = [
    {
      value: 'spam',
      label: t('Spam'),
      examples:
        'Adverts, scams, the same thing posted again and again, or an account run by a bot.',
    },
    {
      value: 'serious-harm',
      label: t('Serious harm'),
      examples:
        "Threats, harassment, hate, sharing someone's private details, or anything that puts someone in danger.",
    },
  ];
</script>

<OverlaySheet
  {open}
  title={sent ? t('Report sent') : `Report ${itemLabel}`}
  labelledById="report-composer-title"
  rootClass="report-composer"
  elevated
  {onClose}
>
  {#if sent}
    <div class="report-sent" role="status">
      <span class="report-sent-icon" aria-hidden="true"
        ><svg viewBox="0 0 24 24"
          ><path
            d="m5 12 5 5 9-10"
            fill="none"
            stroke="currentColor"
            stroke-width="2.4"
            stroke-linecap="round"
            stroke-linejoin="round"
          /></svg
        ></span
      >
      <h3>{t("Thanks, it's under review")}</h3>
      <p>
        {t(
          'Members now review it. If they agree, it\'s hidden for everyone and the author is told why. Until then it carries an "Under review" label.',
        )}
      </p>
    </div>
  {:else}
    <div class="report-body">
      {#if error}<div class="warning-card" role="alert">{error}</div>{/if}
      <div class="report-reasons" role="radiogroup" aria-label={t('Reason')}>
        <p class="report-question">{t('Why are you reporting it?')}</p>
        {#each REASONS as r (r.value)}
          <label class="report-reason" class:on={reason === r.value}>
            <input type="radio" name="report-reason" value={r.value} bind:group={reason} />
            <span><b>{t(r.label)}</b><span>{r.examples}</span></span>
          </label>
        {/each}
      </div>
      <label class="field-stack">
        <span class="field-label">{t("What's wrong?")}</span>
        <textarea bind:value={description} placeholder={t('Say what you saw...')} rows="3"
        ></textarea>
      </label>
    </div>
  {/if}
  {#snippet footer()}
    <div class="sheet-actions">
      {#if sent}
        <button class="sheet-submit report-done" type="button" onclick={onClose}>{t('Done')}</button
        >
      {:else}
        <button
          class="sheet-submit"
          disabled={pending || !description.trim()}
          type="button"
          onclick={onSubmit}
        >
          {pending ? t('Sending…') : t('Send report')}
        </button>
      {/if}
    </div>
  {/snippet}
</OverlaySheet>

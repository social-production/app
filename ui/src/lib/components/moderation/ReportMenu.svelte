<script lang="ts">
  import { t } from '#lib/i18n/index.ts';
  import { portal } from '#lib/components/actions/portal.ts';
  import {
    moderationStatusLabel,
    reasonLabel,
    resolutionLabel,
    thresholdLines,
    voteCountLine,
    type ContentReport,
    type ModerationState,
  } from '#lib/domain/moderation.ts';

  import '#lib/components/styles/report.css';

  /**
   * web's ReportMenu: the ⋮ trigger (with "Under review" / "Hidden" beside it while a report
   * is open) and its dialog: Copy link and Report, or the open report with its thresholds and
   * the No / Yes votes. The parent decides what opening and voting do.
   */
  interface Props {
    itemLabel: string;
    moderation: ModerationState;
    report: ContentReport | null;
    open: boolean;
    pending: boolean;
    copyUrl: string;
    /** Your own item: no Report row (owner, 2026-11-16: no dead "You can't report yourself"). */
    blocked: boolean;
    loading: boolean;
    /** Why the last report or vote was refused (web toasts it). */
    error?: string;
    onToggle: () => void;
    onClose: () => void;
    /** Copies the link; true when it worked. */
    onCopy: () => Promise<boolean>;
    onCompose: () => void;
    onVote: (yes: boolean) => void;
  }

  let {
    itemLabel,
    moderation,
    report,
    open,
    pending,
    copyUrl,
    blocked,
    loading,
    error = '',
    onToggle,
    onClose,
    onCopy,
    onCompose,
    onVote,
  }: Props = $props();

  let copyLabel = $state('Copy link');
  const status = $derived(moderationStatusLabel(moderation, report));
  const canVote = $derived(
    !!report && report.resolution !== 'removed' && report.resolution !== 'dismissed',
  );
  const triggerLabel = $derived(
    `${status ? `${status} · ` : ''}${report || status ? `View ${itemLabel} report` : `Report ${itemLabel}`}`,
  );
  const title = $derived(report ? resolutionLabel(report.resolution) : 'Options');

  $effect(() => {
    if (!open) copyLabel = 'Copy link';
  });

  /** web shows a "Link copied" toast; the app has no toasts, so the item says it. */
  async function copy(): Promise<void> {
    copyLabel = (await onCopy()) ? 'Link copied' : 'Could not copy the link';
    setTimeout(onClose, 800);
  }

  function trigger(event: MouseEvent): void {
    event.preventDefault();
    event.stopPropagation();
    onToggle();
  }

  const canShare = typeof navigator !== 'undefined' && typeof navigator.share === 'function';

  function share(): void {
    void navigator.share({ url: copyUrl }).then(onClose, () => undefined);
  }

  function backdrop(event: MouseEvent): void {
    if (event.target === event.currentTarget) onClose();
  }

  function keydown(event: KeyboardEvent): void {
    if (open && event.key === 'Escape') onClose();
  }
</script>

<svelte:window onkeydown={keydown} />

<div class="report-ui">
  <div class="report-menu-shell">
    <button
      aria-expanded={open}
      aria-label={triggerLabel}
      title={triggerLabel}
      class="report-trigger"
      class:active-report={!!report || !!status}
      class:under-review={status === 'Under review'}
      class:hidden-state={status === 'Hidden'}
      type="button"
      onclick={trigger}
    >
      {#if status}<span class="status-token">{status}</span>{/if}
      <span aria-hidden="true" class="menu-dots"></span>
    </button>
  </div>
</div>

{#if open}
  <div class="report-ui" use:portal>
    <div class="report-menu-backdrop" role="presentation" onclick={backdrop}>
      <div
        aria-label={title}
        aria-modal="true"
        class="report-menu"
        class:action-sheet={!report}
        role="dialog"
        tabindex="-1"
      >
        {#if report}
          <header class="menu-header">
            <h2 class="menu-title">{title}</h2>
          </header>
        {/if}
        <div class="menu-body">
          {#if error}<p class="menu-label" role="alert">{error}</p>{/if}
          {#if copyUrl && !report}
            <button class="menu-item" role="menuitem" type="button" onclick={() => void copy()}>
              <svg aria-hidden="true" viewBox="0 0 24 24"
                ><path
                  d="M10 14a4 4 0 0 0 5.7 0l3-3a4 4 0 0 0-5.7-5.7l-1 1M14 10a4 4 0 0 0-5.7 0l-3 3a4 4 0 0 0 5.7 5.7l1-1"
                  fill="none"
                  stroke="currentColor"
                  stroke-width="1.8"
                  stroke-linecap="round"
                /></svg
              >
              <span>{copyLabel}</span>
            </button>
            {#if canShare}
              <button class="menu-item" role="menuitem" type="button" onclick={share}>
                <svg aria-hidden="true" viewBox="0 0 24 24"
                  ><path
                    d="M12 15V4M8 8l4-4 4 4M5 13v6h14v-6"
                    fill="none"
                    stroke="currentColor"
                    stroke-width="1.8"
                    stroke-linecap="round"
                    stroke-linejoin="round"
                  /></svg
                >
                <span>{t('Share')}</span>
              </button>
            {/if}
          {/if}
          {#if report}
            <div class="review-field">
              <span class="field-label">{t('Reason')}</span>
              <p class="review-value">{reasonLabel(report.reason)}</p>
            </div>
            <div class="review-field">
              <span class="field-label">{t('Description')}</span>
              <p class="report-message">
                {report.description.trim() || 'No additional message was provided.'}
              </p>
            </div>
            <div class="review-meta">
              {#each thresholdLines(report) as line (line)}<p>{line}</p>{/each}
              <p>{voteCountLine(report)}</p>
            </div>
          {:else if loading}
            <p class="menu-label">{t('Loading…')}</p>
          {:else if !blocked}
            <button class="menu-item menu-danger" role="menuitem" type="button" onclick={onCompose}>
              <svg aria-hidden="true" viewBox="0 0 24 24"
                ><path
                  d="M6 21V4M6 4h11l-2.5 4.5L17 13H6"
                  fill="none"
                  stroke="currentColor"
                  stroke-width="1.8"
                  stroke-linecap="round"
                  stroke-linejoin="round"
                /></svg
              >
              <span>Report {itemLabel}</span>
            </button>
          {/if}
        </div>
        <footer class="menu-actions" class:votes={canVote}>
          <button aria-label={t('Close')} class="menu-dismiss" type="button" onclick={onClose}
            >×</button
          >
          {#if canVote && report}
            <button
              class="vote-chip"
              class:active-vote={report.activeVote === 'no'}
              disabled={pending}
              type="button"
              onclick={() => onVote(false)}>{t('No')}</button
            >
            <button
              class="vote-chip yes"
              class:active-vote={report.activeVote === 'yes'}
              disabled={pending}
              type="button"
              onclick={() => onVote(true)}>{t('Yes')}</button
            >
          {/if}
        </footer>
      </div>
    </div>
  </div>
{/if}

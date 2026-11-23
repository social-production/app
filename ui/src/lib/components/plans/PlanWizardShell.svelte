<script lang="ts">
  import { t } from '#lib/i18n/index.ts';
  import type { Snippet } from 'svelte';

  import { fitText } from '#lib/components/actions/fitText.ts';
  import { portal } from '#lib/components/actions/portal.ts';

  import '#lib/components/styles/wizard.css';

  /** web's PlanWizardShell: a step-by-step dialog, full screen on phones, in the create flows'
   * look (Owner Notes Mockups §4): plain ×, step bars, and Back | Next joined in the dock. */
  interface Props {
    open: boolean;
    title: string;
    stepIndex: number;
    stepCount: number;
    nextLabel: string;
    canGoBack: boolean;
    canGoNext: boolean;
    /** web's `showFooter`: off when the step has its own actions (a vote); phones keep ×. */
    showFooter?: boolean;
    onClose: () => void;
    /** Backdrop or Escape: hide, keeping the draft and step. */
    onDismiss: () => void;
    onBack: () => void;
    onNext: () => void;
    children: Snippet;
  }

  let {
    open,
    title,
    stepIndex,
    stepCount,
    nextLabel,
    canGoBack,
    canGoNext,
    showFooter = true,
    onClose,
    onDismiss,
    onBack,
    onNext,
    children,
  }: Props = $props();

  let compact = $state(false);
  let body = $state<HTMLDivElement>();

  $effect(() => {
    const media = window.matchMedia('(max-width: 760px)');
    const sync = (): void => {
      compact = media.matches;
    };
    sync();
    media.addEventListener('change', sync);
    return () => media.removeEventListener('change', sync);
  });

  // Each step opens at its top.
  $effect(() => {
    void stepIndex;
    body?.scrollTo({ top: 0, behavior: 'instant' });
  });

  function keydown(event: KeyboardEvent): void {
    if (event.key === 'Escape') onDismiss();
  }
</script>

{#if open}
  <div
    class="wizard-backdrop"
    class:compact
    role="presentation"
    use:portal
    onclick={(e) => e.target === e.currentTarget && onDismiss()}
    onkeydown={keydown}
  >
    <div class="wizard-shell" class:compact role="dialog" aria-modal="true" aria-label={title}>
      <!-- The create flows' top bar (Owner Notes Mockups §4): a plain × on the left, the title
           centred with its step under it, then one short bar per step. -->
      <header class="wizard-header wizard-topbar" class:compact>
        <button aria-label={t('Close')} class="wizard-x" type="button" onclick={onClose}>×</button>
        <div class="wizard-heading">
          <span class="wizard-title" use:fitText={{ min: 11, text: title }}>{title}</span>
          {#if stepCount > 1}
            <span class="wizard-step-count"
              >{t('Step {n} of {total}', { n: stepIndex + 1, total: stepCount })}</span
            >
          {/if}
        </div>
        {#if stepCount > 1}
          <div class="wizard-step-bars" aria-hidden="true">
            {#each Array.from({ length: stepCount }, (_, i) => i) as i (i)}<i
                class:on={i <= stepIndex}
              ></i>{/each}
          </div>
        {/if}
      </header>

      <div class="wizard-body" bind:this={body}>{@render children()}</div>

      {#if showFooter}
        <footer class="wizard-footer">
          {#if showFooter && canGoBack}
            <button class="secondary-button" type="button" onclick={onBack}>{t('Back')}</button>
          {/if}
          {#if showFooter}
            <button class="primary-button" disabled={!canGoNext} type="button" onclick={onNext}>
              {t(nextLabel)}
            </button>
          {/if}
        </footer>
      {/if}
    </div>
  </div>
{/if}

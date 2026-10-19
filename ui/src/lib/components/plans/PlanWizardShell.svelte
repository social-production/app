<script lang="ts">
  import type { Snippet } from 'svelte';

  import { fitText } from '#lib/components/actions/fitText.ts';
  import { portal } from '#lib/components/actions/portal.ts';

  import '#lib/components/styles/wizard.css';

  /** web's PlanWizardShell: a step-by-step dialog, full screen on phones, with a progress bar. */
  interface Props {
    open: boolean;
    title: string;
    stepIndex: number;
    stepCount: number;
    nextLabel: string;
    canGoBack: boolean;
    canGoNext: boolean;
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
    onClose,
    onDismiss,
    onBack,
    onNext,
    children,
  }: Props = $props();

  let compact = $state(false);
  let body = $state<HTMLDivElement>();

  const progress = $derived(stepCount > 0 ? Math.round(((stepIndex + 1) / stepCount) * 100) : 0);

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
      <header class="wizard-header" class:compact>
        <div class="wizard-header-main">
          <span class="wizard-step-number">{stepIndex + 1}/{stepCount}</span>
          <span class="wizard-title" use:fitText={{ min: 11, text: title }}>{title}</span>
        </div>
        {#if !compact}
          <button aria-label="Close" class="cancel-button" type="button" onclick={onClose}>×</button
          >
        {/if}
        <div class="progress-track" aria-hidden="true">
          <span class="progress-fill" style={`width: ${progress}%`}></span>
        </div>
      </header>

      <div class="wizard-body" bind:this={body}>{@render children()}</div>

      <footer class="wizard-footer">
        {#if compact}
          <button
            aria-label="Close"
            class="secondary-button wizard-close"
            type="button"
            onclick={onClose}>×</button
          >
        {/if}
        {#if canGoBack}
          <button class="secondary-button" type="button" onclick={onBack}>Back</button>
        {/if}
        <button class="primary-button" disabled={!canGoNext} type="button" onclick={onNext}>
          {nextLabel}
        </button>
      </footer>
    </div>
  </div>
{/if}

<script lang="ts">
  import type { Snippet } from 'svelte';

  import OverlaySheet from '#lib/components/overlay/OverlaySheet.svelte';

  import '#lib/components/styles/create.css';

  /**
   * web's CreateFlowLayout + CreateWizard: a wide sheet with a step rail, the current step,
   * and Back/Continue (Cancel/Publish at the ends). Closing with typed text asks first.
   */
  interface Props {
    title: string;
    description: string;
    steps: { id: string; title: string }[];
    stepIndex: number;
    /** Empty required fields on this step; Continue waits and a line names them. */
    missing: string[];
    submitLabel: string;
    submitting: boolean;
    onStep: (index: number) => void;
    onSubmit: () => void;
    onClose: () => void;
    children: Snippet;
  }

  let p: Props = $props();

  let fields = $state<HTMLDivElement>();
  let confirming = $state(false);

  const last = $derived(p.stepIndex === p.steps.length - 1);

  function hasTypedText(): boolean {
    const inputs = fields?.querySelectorAll<HTMLInputElement | HTMLTextAreaElement>(
      'textarea, input:not([type]), input[type="text"], input[type="search"]',
    );
    return [...(inputs ?? [])].some((field) => field.value.trim().length > 0);
  }

  function close(): void {
    // Don't throw away what someone typed without asking.
    if (hasTypedText()) confirming = true;
    else p.onClose();
  }

  function go(index: number): void {
    p.onStep(Math.max(0, Math.min(index, p.steps.length - 1)));
    document.querySelector('.overlay-sheet .overlay-body')?.scrollTo({ top: 0 });
  }
</script>

<OverlaySheet
  open
  title={p.title}
  labelledById="create-sheet-title"
  wide
  rootClass="create-sheet"
  onClose={close}
>
  {#snippet subtitle()}
    {#if p.stepIndex === 0}<p class="sheet-description">{p.description}</p>{/if}
  {/snippet}
  {#snippet toolbar()}
    <nav class="step-rail" aria-label="Create steps">
      {#each p.steps as step, index (step.id)}
        <button
          class="step-chip"
          class:active={index === p.stepIndex}
          class:complete={index < p.stepIndex}
          type="button"
          onclick={() => go(index)}>{step.title}</button
        >
      {/each}
    </nav>
  {/snippet}
  <div class="sheet-form">
    <div class="sheet-fields" bind:this={fields}>
      <section class="panel bare">
        <div class="body">
          <div class="wizard">
            <div class="step-body">{@render p.children()}</div>
            {#if p.missing.length}
              <p class="missing-note" role="status">Still needed: {p.missing.join(', ')}.</p>
            {/if}
          </div>
        </div>
      </section>
    </div>
  </div>
  {#snippet footer()}
    <div class="sheet-actions">
      <button
        class="sheet-cancel"
        type="button"
        onclick={() => (p.stepIndex === 0 ? close() : go(p.stepIndex - 1))}
        >{p.stepIndex === 0 ? 'Cancel' : 'Back'}</button
      >
      <button
        class="sheet-submit"
        type="button"
        disabled={p.missing.length > 0 || p.submitting}
        onclick={() => (last ? p.onSubmit() : go(p.stepIndex + 1))}
        >{last ? (p.submitting ? 'Creating...' : p.submitLabel) : 'Continue'}</button
      >
    </div>
  {/snippet}
</OverlaySheet>

<OverlaySheet
  open={confirming}
  title="Discard this draft?"
  labelledById="discard-draft"
  rootClass="create-sheet"
  onClose={() => (confirming = false)}
>
  <p class="sheet-description">What you've written here will be lost.</p>
  {#snippet footer()}
    <div class="sheet-actions">
      <button class="sheet-cancel" type="button" onclick={() => (confirming = false)}
        >Keep editing</button
      >
      <button class="sheet-submit" type="button" onclick={() => ((confirming = false), p.onClose())}
        >Discard</button
      >
    </div>
  {/snippet}
</OverlaySheet>

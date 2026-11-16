<script lang="ts">
  import type { Snippet } from 'svelte';

  import OverlaySheet from '#lib/components/overlay/OverlaySheet.svelte';

  import '#lib/components/styles/create.css';

  /**
   * web's CreateFlowLayout + CreateWizard (Polish Pass mockups, 2026-11-16): the close button at
   * the top left, the title with "Step 1 of 4" under it over one progress line, each step's
   * question as its heading, and Back / Continue at the bottom. Continue stays faded until the
   * step is complete; tapping it then says what is missing and goes to that field. Closing
   * with typed text asks first.
   */
  interface Props {
    title: string;
    description: string;
    /** `question` heads the step ("Where does it happen?"). */
    steps: { id: string; title: string; question?: string }[];
    stepIndex: number;
    /** Empty required fields on this step; Continue waits and a line names them. */
    missing: string[];
    submitLabel: string;
    /** web's `submittingLabel` ("Posting..." for posts). */
    submittingLabel?: string;
    /** One-step forms (web's non-wizard CreateFlowLayout) just disable Publish. */
    canSubmit?: boolean;
    submitting: boolean;
    onStep: (index: number) => void;
    onSubmit: () => void;
    onClose: () => void;
    children: Snippet;
  }

  let p: Props = $props();

  let fields = $state<HTMLDivElement>();
  let confirming = $state(false);
  // Set when someone taps a faded Continue: the hint shows until the step changes.
  let tried = $state(false);

  const last = $derived(p.stepIndex === p.steps.length - 1);
  const stepped = $derived(p.steps.length > 1);
  const question = $derived(p.steps[p.stepIndex]?.question ?? '');
  const blocked = $derived(p.missing.length > 0 || p.canSubmit === false);
  // "Help request title is required." reads "help request title" in the hint.
  const hint = $derived(
    p.missing
      .map((m) => m.replace(/\.$/, '').replace(/ is required$/i, ''))
      .map((m) => m.charAt(0).toLowerCase() + m.slice(1))
      .join(', '),
  );

  function advance(): void {
    if (p.submitting) return;
    if (blocked) {
      tried = true;
      const required = fields?.querySelectorAll<HTMLInputElement | HTMLTextAreaElement>(
        'input[aria-required="true"], textarea[aria-required="true"]',
      );
      const empty = [...(required ?? [])].find((field) => !field.value.trim());
      empty?.focus();
      return;
    }
    if (last) p.onSubmit();
    else go(p.stepIndex + 1);
  }

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
    tried = false;
    p.onStep(Math.max(0, Math.min(index, p.steps.length - 1)));
    document.querySelector('.overlay-sheet .overlay-body')?.scrollTo({ top: 0 });
  }
</script>

<OverlaySheet
  open
  title={p.title}
  labelledById="create-sheet-title"
  wide
  rootClass="create-sheet create-main"
  onClose={close}
>
  {#snippet headerActions()}
    <button class="create-close" type="button" aria-label="Close" title="Close" onclick={close}
      >×</button
    >
  {/snippet}
  {#snippet subtitle()}
    {#if stepped}<p class="sheet-description step-count">
        Step {p.stepIndex + 1} of {p.steps.length}
      </p>{/if}
  {/snippet}
  {#snippet toolbar()}
    {#if stepped}<div class="step-line" aria-hidden="true">
        <i style={`width: ${((p.stepIndex + 1) / p.steps.length) * 100}%`}></i>
      </div>{/if}
  {/snippet}
  <div class="sheet-form">
    <div class="sheet-fields" bind:this={fields}>
      <section class="panel bare">
        <div class="body">
          <div class="wizard">
            {#if question}<h3 class="step-question">{question}</h3>
            {:else if !stepped && p.description}<p class="step-intro">{p.description}</p>{/if}
            <div class="step-body">{@render p.children()}</div>
            {#if tried && hint}
              <p class="missing-note" role="status">To continue, add: {hint}.</p>
            {/if}
          </div>
        </div>
      </section>
    </div>
  </div>
  {#snippet footer()}
    <div class="sheet-actions">
      {#if p.stepIndex > 0}
        <button class="sheet-cancel create-back" type="button" onclick={() => go(p.stepIndex - 1)}
          >Back</button
        >
      {/if}
      <button
        class="sheet-submit"
        class:faded={blocked}
        type="button"
        aria-disabled={blocked}
        disabled={p.submitting}
        onclick={advance}
        >{last
          ? p.submitting
            ? (p.submittingLabel ?? 'Creating...')
            : p.submitLabel
          : 'Continue'}</button
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

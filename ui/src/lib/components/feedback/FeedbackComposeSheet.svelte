<script lang="ts">
  import ToolbarIcon from '#lib/components/icons/ToolbarIcon.svelte';
  import OverlaySheet from '#lib/components/overlay/OverlaySheet.svelte';
  import {
    FEEDBACK_DESCRIPTION_MAX,
    FEEDBACK_TITLE_MAX,
    type FeedbackInput,
    type FeedbackKind,
  } from '#lib/domain/feedback.ts';

  import '#lib/components/styles/feedback.css';

  /** web's FeedbackComposeSheet: type, title, description. */
  interface Props {
    open: boolean;
    pending: boolean;
    message: string;
    onSubmit: (input: FeedbackInput) => void;
    onClose: () => void;
  }

  let { open, pending, message, onSubmit, onClose }: Props = $props();

  const kinds: { value: FeedbackKind; label: string; icon: string }[] = [
    { value: 'bug', label: 'Bug', icon: 'bug' },
    { value: 'suggestion', label: 'Suggestion', icon: 'lightbulb' },
  ];
  let kind = $state<FeedbackKind>('bug');
  let title = $state('');
  let description = $state('');

  // Each opening starts empty.
  $effect(() => {
    if (open) {
      kind = 'bug';
      title = '';
      description = '';
    }
  });

  function submit(event: SubmitEvent): void {
    event.preventDefault();
    onSubmit({ kind, title: title.trim(), description: description.trim() });
  }
</script>

<OverlaySheet
  {open}
  title="Add feedback"
  labelledById="feedback-compose-sheet"
  rootClass="feedback-sheet"
  wide
  {onClose}
>
  <form id="feedback-compose-form" class="feedback-form" onsubmit={submit}>
    {#if message}<div class="warning-card" role="alert">{message}</div>{/if}
    <div class="field">
      <span class="field-label">Type</span>
      <div class="type-switcher" role="radiogroup" aria-label="Feedback type">
        {#each kinds as option (option.value)}
          <button
            aria-checked={kind === option.value}
            class="type-option"
            class:active={kind === option.value}
            role="radio"
            type="button"
            onclick={() => (kind = option.value)}
          >
            <ToolbarIcon name={option.icon} />
            <span>{option.label}</span>
          </button>
        {/each}
      </div>
    </div>
    <label class="field">
      <span class="field-label">Title</span>
      <input
        bind:value={title}
        maxlength={FEEDBACK_TITLE_MAX}
        placeholder="Summarize the bug or suggestion"
        type="text"
      />
    </label>
    <label class="field">
      <span class="field-label">Description</span>
      <textarea
        bind:value={description}
        maxlength={FEEDBACK_DESCRIPTION_MAX}
        placeholder="Describe what happened, what you expected, or how the platform could improve."
        rows="7"></textarea>
    </label>
  </form>
  {#snippet footer()}
    <div class="sheet-actions">
      <button class="sheet-cancel" type="button" onclick={onClose}>Cancel</button>
      <button
        class="sheet-submit"
        disabled={pending || !title.trim() || !description.trim()}
        form="feedback-compose-form"
        type="submit">{pending ? 'Submitting…' : 'Submit feedback'}</button
      >
    </div>
  {/snippet}
</OverlaySheet>

<script lang="ts">
  import type { CharterDraft } from '#lib/domain/projects.ts';

  import OverlaySheet from './OverlaySheet.svelte';

  /**
   * web's ProposeEditSheet: title, description, location label, filed as a proposal (M3).
   * Organizer-run events save directly ("Edit event", ADR 0061).
   */
  interface Props {
    open: boolean;
    sheetTitle?: string;
    submitLabel?: string;
    appliesDirectly?: boolean;
    draft: CharterDraft;
    pending: boolean;
    message: string | undefined;
    onSubmit: (draft: CharterDraft) => void;
    onClose: () => void;
  }

  let {
    open,
    sheetTitle = 'Propose Edit',
    submitLabel = 'Propose Edit',
    appliesDirectly = false,
    draft,
    pending,
    message,
    onSubmit,
    onClose,
  }: Props = $props();

  let title = $state('');
  let description = $state('');
  let location = $state('');

  // Each opening starts from the current values.
  $effect(() => {
    if (open) ({ title, description, location } = draft);
  });

  function submit(event: SubmitEvent): void {
    event.preventDefault();
    onSubmit({ title: title.trim(), description, location: location.trim() });
  }
</script>

<OverlaySheet
  {open}
  title={sheetTitle}
  labelledById="project-edit-sheet"
  rootClass="edit-sheet"
  {onClose}
>
  {#snippet subtitle()}
    <p class="sheet-lead">
      Change the title, description, or location.
      {appliesDirectly
        ? 'It applies as soon as you save.'
        : 'Members vote on it, and your proposal counts as your Yes. While few members are active, your Yes alone can be enough, and then it applies straight away.'}
    </p>
  {/snippet}
  <form id="project-edit-sheet-form" class="edit-form" onsubmit={submit}>
    {#if message}<div class="warning-card" role="alert">{message}</div>{/if}
    <label class="field">
      <span class="field-label">Title</span>
      <input bind:value={title} maxlength="120" placeholder="Project title" />
    </label>
    <label class="field">
      <span class="field-label">Description</span>
      <textarea bind:value={description} rows="6" placeholder="Describe the project..."></textarea>
    </label>
    <label class="field">
      <span class="field-label">Location</span>
      <input bind:value={location} maxlength="160" placeholder="Town, region or Online" />
    </label>
  </form>
  {#snippet footer()}
    <div class="sheet-actions">
      <button class="sheet-cancel" type="button" onclick={onClose}>Cancel</button>
      <button
        class="sheet-submit"
        disabled={pending || !title.trim() || !description.trim()}
        form="project-edit-sheet-form"
        type="submit">{pending ? 'Working...' : submitLabel}</button
      >
    </div>
  {/snippet}
</OverlaySheet>

<script lang="ts">
  import { t } from '#lib/i18n/index.ts';
  import LocationPicker from '#lib/components/location/LocationPicker.svelte';
  import { emptyPlaceValue, type PlaceSearch, type PlaceValue } from '#lib/domain/places.ts';
  import { charterDraft, type Charter, type CharterDraft } from '#lib/domain/projects.ts';

  import type { Snippet } from 'svelte';

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
    /** The charter as it is; each opening starts from it. */
    charter: Charter;
    pending: boolean;
    message: string | undefined;
    onSubmit: (draft: CharterDraft) => void;
    onClose: () => void;
    places: PlaceSearch;
    /** The cover and picture with camera buttons, above the fields (ADR 0117). */
    art?: Snippet | undefined;
  }

  let {
    open,
    sheetTitle = 'Propose Edit',
    submitLabel = 'Propose Edit',
    appliesDirectly = false,
    charter,
    pending,
    message,
    onSubmit,
    onClose,
    places,
    art,
  }: Props = $props();

  let title = $state('');
  let description = $state('');
  let location = $state<PlaceValue>(emptyPlaceValue());

  // Each opening starts from the current values.
  $effect(() => {
    if (open) ({ title, description, location } = charterDraft(charter));
  });

  function submit(event: SubmitEvent): void {
    event.preventDefault();
    onSubmit({ title: title.trim(), description, location });
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
      {art
        ? t('Change the picture, cover, title, description, or location.')
        : t('Change the title, description, or location.')}
      {appliesDirectly
        ? t('It applies as soon as you save.')
        : t(
            'Members vote on it, and your proposal counts as your Yes. While few members are active, your Yes alone can be enough, and then it applies straight away.',
          )}
    </p>
  {/snippet}
  <form id="project-edit-sheet-form" class="edit-form" onsubmit={submit}>
    {#if message}<div class="warning-card" role="alert">{message}</div>{/if}
    {@render art?.()}
    <label class="field">
      <span class="field-label">{t('Title')}</span>
      <input bind:value={title} maxlength="120" placeholder={t('Project title')} />
    </label>
    <label class="field">
      <span class="field-label">{t('Description')}</span>
      <textarea bind:value={description} rows="6" placeholder={t('Describe the project...')}
      ></textarea>
    </label>
    <div class="field">
      <span class="field-label">{t('Location')}</span>
      <LocationPicker
        value={location}
        portaled
        elevated
        modes={['physical', 'online']}
        onChange={(next) => (location = next)}
        search={places.search}
        failure={places.failure}
      />
    </div>
  </form>
  {#snippet footer()}
    <div class="sheet-actions">
      <button class="sheet-cancel" type="button" onclick={onClose}>{t('Cancel')}</button>
      <button
        class="sheet-submit"
        disabled={pending || !title.trim() || !description.trim()}
        form="project-edit-sheet-form"
        type="submit">{pending ? t('Working...') : submitLabel}</button
      >
    </div>
  {/snippet}
</OverlaySheet>

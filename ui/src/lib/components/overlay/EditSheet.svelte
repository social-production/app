<script lang="ts">
  import { t } from '#lib/i18n/index.ts';
  import CreateSheet from '#lib/components/create/CreateSheet.svelte';
  import LocationIntent from '#lib/components/create/LocationIntent.svelte';
  import RequiredLabel from '#lib/components/create/RequiredLabel.svelte';
  import { onlinePlaceValue, type PlaceSearch, type PlaceValue } from '#lib/domain/places.ts';
  import {
    charterDraft,
    type Charter,
    type CharterDraft,
    type LocationChoice,
  } from '#lib/domain/projects.ts';

  import type { Snippet } from 'svelte';

  import '#lib/components/styles/ux/editsheet.css';

  /**
   * web's ProposeEditSheet in the create flow's look (Edit, Plan and Meetup mockups §A, owner
   * 2026-11-24): the cover and picture on top, filled fields, the place as In person / Online,
   * and a "Changed" tag on each field that differs. Filed as a proposal (M3); organizer-run
   * events save directly ("Edit event", ADR 0061).
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
  let location = $state<PlaceValue>(onlinePlaceValue());
  const before = $derived(charterDraft(charter));
  const intent = $derived<LocationChoice>(location.mode === 'online' ? 'online' : 'physical');
  const missing = $derived(
    [!title.trim() && 'Title', !description.trim() && 'Description'].filter(
      (m): m is string => !!m,
    ),
  );

  // Each opening starts from the current values.
  $effect(() => {
    if (open) ({ title, description, location } = charterDraft(charter));
  });

  function setIntent(next: LocationChoice): void {
    if (next === 'online') location = onlinePlaceValue();
    else if (location.mode === 'online') location = { ...before.location, mode: 'physical' };
  }
</script>

{#snippet changed(differs: boolean)}
  {#if differs}<em class="field-changed">{t('Changed')}</em>{/if}
{/snippet}

{#if open}
  <CreateSheet
    title={t(sheetTitle)}
    description={appliesDirectly
      ? t('It applies as soon as you save.')
      : t(
          'Members vote on it, and your proposal counts as your Yes. While few members are active, your Yes alone can be enough, and then it applies straight away.',
        )}
    steps={[{ id: 'edit', title: sheetTitle }]}
    stepIndex={0}
    {missing}
    submitLabel={t(submitLabel)}
    submittingLabel={t('Working...')}
    submitting={pending}
    askBeforeClose={false}
    onStep={() => {}}
    onSubmit={() => onSubmit({ title: title.trim(), description, location })}
    {onClose}
  >
    <div class="form-stack edit-form">
      {#if message}<div class="warning-card" role="alert">{message}</div>{/if}
      {@render art?.()}
      <label>
        <span class="edit-label"
          ><RequiredLabel>{t('Title')}</RequiredLabel>{@render changed(
            title.trim() !== before.title.trim(),
          )}</span
        >
        <input bind:value={title} maxlength="120" aria-required="true" />
      </label>
      <label>
        <span class="edit-label"
          ><RequiredLabel>{t('Description')}</RequiredLabel>{@render changed(
            description.trim() !== before.description.trim(),
          )}</span
        >
        <textarea bind:value={description} rows="5" aria-required="true"></textarea>
      </label>
      <div class="edit-place">
        <span class="edit-label"
          >{t('Location')}{@render changed(
            location.displayLabel !== before.location.displayLabel ||
              location.mode !== before.location.mode,
          )}</span
        >
        <LocationIntent
          {intent}
          place={location}
          allowLater={false}
          search={places.search}
          failure={places.failure}
          onIntent={setIntent}
          onPlace={(next) => (location = next)}
        />
      </div>
    </div>
  </CreateSheet>
{/if}

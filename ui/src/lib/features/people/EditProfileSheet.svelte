<script lang="ts">
  import AvatarBadge from '#lib/components/icons/AvatarBadge.svelte';
  import OverlaySheet from '#lib/components/overlay/OverlaySheet.svelte';
  import { HANDLE_MAX, handleProblem } from '#lib/domain/handle.ts';
  import ToolbarIcon from '#lib/components/icons/ToolbarIcon.svelte';
  import { devicePositionMessage } from '#lib/domain/defaultLocation.ts';
  import { BIO_MAX, HOME_AREA_MAX, type Person } from '#lib/domain/people.ts';
  import { devicePlace } from '#lib/services/locations.ts';

  import type { ProfileEditor } from './profileEdit.svelte.ts';

  /**
   * Edit profile, opened over your profile (Profiles and Comments mockups §2): the cover and the
   * photo with their camera buttons, then name, about and area. Settings keeps the same fields.
   */
  let { editor, person }: { editor: ProfileEditor; person: Person } = $props();

  let name = $state('');
  let bio = $state('');
  let area = $state('');
  let locating = $state(false);
  let areaNote = $state('');

  /** The Area field's locate button (Profiles mockups §2): this device's place, once. */
  async function locate(): Promise<void> {
    locating = true;
    areaNote = '';
    const result = await devicePlace();
    locating = false;
    if (!result.ok) {
      areaNote = devicePositionMessage(result.error);
      return;
    }
    // "76, Greeves Street, Fitzroy, Melbourne, Victoria, 3065, Australia" → "Fitzroy, Melbourne":
    // no numbers, no state or country, the two smallest places left.
    const parts = result.place.displayLabel
      .split(',')
      .map((p) => p.trim())
      .filter((p) => p && !/^\d/.test(p));
    const local = parts.length > 3 ? parts.slice(0, -2) : parts;
    area = local.slice(-2).join(', ') || result.place.displayLabel;
  }
  let coverInput = $state<HTMLInputElement>();
  let photoInput = $state<HTMLInputElement>();
  const problem = $derived(handleProblem(name));

  $effect(() => {
    if (!editor.open) return;
    name = person.profile?.name ?? '';
    bio = person.profile?.bio ?? '';
    area = person.profile?.home_area ?? '';
  });

  const picked = (event: Event): File | undefined =>
    (event.currentTarget as HTMLInputElement).files?.[0];

  async function save(): Promise<void> {
    const done = await editor.run(() =>
      editor.save(person, { name: name.trim(), bio, home_area: area.trim() }),
    );
    if (done) editor.open = false;
  }
</script>

<OverlaySheet
  open={editor.open}
  title="Edit profile"
  labelledById="edit-profile-title"
  rootClass="edit-profile-sheet"
  onClose={() => (editor.open = false)}
>
  <div class="edit-profile">
    <div class="edit-cover" style:background-image={editor.cover ? `url(${editor.cover})` : null}>
      <button class="edit-cam wide" type="button" onclick={() => coverInput?.click()}>
        <svg aria-hidden="true" viewBox="0 0 24 24"
          ><rect x="3" y="5" width="18" height="14" rx="2" /><circle cx="9" cy="10" r="2" /><path
            d="m21 16-5-5-8 8"
          /></svg
        >{editor.cover ? 'Change cover' : 'Add a cover'}</button
      >
      {#if editor.cover}
        <button class="edit-cover-remove" type="button" onclick={() => void editor.removeCover()}
          >Remove</button
        >
      {/if}
    </div>
    <button
      class="edit-avatar"
      type="button"
      aria-label="Change profile photo"
      onclick={() => photoInput?.click()}
    >
      <AvatarBadge
        size="md"
        name={person.profile?.name ?? ''}
        avatar={person.profile?.avatar ?? null}
      />
      <span class="hero-cam" aria-hidden="true"
        ><svg viewBox="0 0 24 24"
          ><rect x="3" y="5" width="18" height="14" rx="2" /><circle cx="9" cy="10" r="2" /></svg
        ></span
      >
    </button>
    <input
      bind:this={coverInput}
      hidden
      type="file"
      accept="image/*"
      aria-label="Cover photo"
      onchange={(e) => {
        const file = picked(e);
        if (file) void editor.pickCover(file);
      }}
    />
    <input
      bind:this={photoInput}
      hidden
      type="file"
      accept="image/*"
      aria-label="Profile photo"
      onchange={(e) => {
        const file = picked(e);
        if (file) void editor.pickPhoto(person, file);
      }}
    />
    <label class="edit-field">
      <span>Name</span>
      <input bind:value={name} maxlength={HANDLE_MAX} autocomplete="name" />
    </label>
    {#if problem}<p class="edit-hint">{problem}</p>{/if}
    <label class="edit-field">
      <span>About you</span>
      <textarea bind:value={bio} maxlength={BIO_MAX} rows="3"></textarea>
    </label>
    <label class="edit-field">
      <span>Area</span>
      <span class="edit-area">
        <input bind:value={area} maxlength={HOME_AREA_MAX} placeholder="e.g. Fitzroy, Melbourne" />
        <button
          class="edit-locate"
          type="button"
          aria-label="Use my location"
          title="Use my location"
          disabled={locating}
          onclick={() => void locate()}><ToolbarIcon name="locate" /></button
        >
      </span>
    </label>
    {#if areaNote}<p class="edit-hint">{areaNote}</p>{/if}
    {#if editor.error}<p class="edit-hint error" role="alert">{editor.error}</p>{/if}
  </div>
  {#snippet footer()}
    <div class="sheet-actions">
      <button
        class="sheet-submit"
        type="button"
        disabled={editor.saving || !!problem}
        onclick={() => void save()}>{editor.saving ? 'Saving…' : 'Save'}</button
      >
    </div>
  {/snippet}
</OverlaySheet>

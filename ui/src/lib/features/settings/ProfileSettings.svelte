<script lang="ts">
  import { cropPhoto } from '#lib/stores/photoCrop.svelte.ts';
  import { t } from '#lib/i18n/index.ts';
  import SkeletonRows from '#lib/components/loading/SkeletonRows.svelte';
  import AvatarBadge from '#lib/components/icons/AvatarBadge.svelte';
  import OverlaySheet from '#lib/components/overlay/OverlaySheet.svelte';
  import SettingRow from '#lib/components/settings/SettingRow.svelte';
  import { HANDLE_MAX, handleProblem } from '#lib/domain/handle.ts';
  import { BIO_MAX, HOME_AREA_MAX, type Avatar } from '#lib/domain/people.ts';
  import { toAvatar } from '#lib/components/people/avatar.ts';
  import { explain } from '#lib/services/errors.ts';
  import { getPerson, saveProfile } from '#lib/services/people.ts';
  import { viewer } from '#lib/stores/viewer.svelte.ts';

  let { me }: { me: string } = $props();

  let name = $state('');
  const problem = $derived(handleProblem(name));
  let bio = $state('');
  let homeArea = $state('');
  let avatar = $state<Avatar | null>(null);
  let status = $state<string>();
  let error = $state<string>();
  let saving = $state(false);
  let loaded = $state(false);
  let photoInput = $state<HTMLInputElement>();
  let editing = $state<'name' | 'area' | 'bio' | null>(null);

  $effect(() => {
    void getPerson(me).then((person) => {
      name = person.profile?.name ?? '';
      bio = person.profile?.bio ?? '';
      homeArea = person.profile?.home_area ?? '';
      avatar = person.profile?.avatar ?? null;
      loaded = true;
    });
  });

  async function pick(event: Event): Promise<void> {
    const input = event.currentTarget as HTMLInputElement;
    const chosen = input.files?.[0];
    input.value = '';
    if (!chosen) return;
    // Placed in its round frame first (Owner Notes Mockups §3).
    const file = await cropPhoto(chosen, 'round');
    if (!file) return;
    error = undefined;
    try {
      avatar = await toAvatar(file);
      const note = t('Picture resized to 256×256 ({kb} KB).', {
        kb: Math.ceil(avatar.bytes.length / 1024),
      });
      status = note;
      await save();
      if (!error) status = `${note} Saved.`;
    } catch (e) {
      error = explain(e);
    }
  }

  async function save(): Promise<void> {
    saving = true;
    error = undefined;
    status = undefined;
    try {
      await saveProfile({ name: name.trim(), bio, home_area: homeArea.trim(), avatar });
      viewer.name = name.trim();
      status = t('Saved.');
    } catch (e) {
      error = explain(e);
    } finally {
      saving = false;
    }
  }
</script>

<section class="settings-section" id="settings-profile">
  {#if !loaded}
    <SkeletonRows count={3} kind="row" />
  {:else}
    <!-- As drawn: who you are on top, then one row per field; a row opens a small sheet. -->
    <div class="profile-head">
      <div class="avatar-wrap">
        <button
          class="avatar-picker"
          type="button"
          aria-label={t('Change profile photo')}
          onclick={() => photoInput?.click()}
        >
          <AvatarBadge size="md" {name} {avatar} />
        </button>
        {#if avatar}
          <button
            class="avatar-remove"
            type="button"
            aria-label={t('Remove photo')}
            onclick={() => {
              avatar = null;
              void save();
            }}>×</button
          >
        {/if}
      </div>
      <input bind:this={photoInput} accept="image/*" class="sr-only" type="file" onchange={pick} />
      <div class="profile-head-copy">
        <strong>{name || 'Unnamed'}</strong>
        <span>
          {#if homeArea}{homeArea} ·{/if}
          <button class="text-link" type="button" onclick={() => photoInput?.click()}
            >{t('Change photo')}</button
          >
        </span>
      </div>
    </div>
    <h2>{t('Profile')}</h2>
    <div class="card stack flush">
      <SettingRow label={t('Username')} value={name} onOpen={() => (editing = 'name')} />
      <SettingRow
        label={t('Home area')}
        value={homeArea || 'Add'}
        onOpen={() => (editing = 'area')}
      />
      <SettingRow
        label={t('About you')}
        value={bio ? `${bio.slice(0, 24)}${bio.length > 24 ? '…' : ''}` : t('Add')}
        onOpen={() => (editing = 'bio')}
      />
    </div>
    <!-- Reserved height: messages appear without pushing the page down. -->
    <p class:profile-image-error={!!error} class:bio-saved={!error} class="status-line">
      {error ?? status ?? ''}
    </p>
  {/if}
</section>

<OverlaySheet
  open={editing !== null}
  title={editing === 'name' ? t('Username') : editing === 'area' ? t('Home area') : t('About you')}
  labelledById="setting-profile-field"
  rootClass="setting-sheet"
  onClose={() => (editing = null)}
>
  <div class="setting-field">
    {#if editing === 'name'}
      <input
        bind:value={name}
        maxlength={HANDLE_MAX}
        aria-label={t('Username')}
        placeholder={t('Username')}
        autocomplete="name"
      />
      {#if problem}<p class="setting-hint">{problem}</p>{/if}
    {:else if editing === 'area'}
      <input
        bind:value={homeArea}
        maxlength={HOME_AREA_MAX}
        aria-label={t('Home area')}
        placeholder={t('Home area, e.g. Brisbane, QLD')}
      />
    {:else if editing === 'bio'}
      <textarea
        bind:value={bio}
        maxlength={BIO_MAX}
        rows="5"
        aria-label={t('About you')}
        placeholder={t('About you')}></textarea>
      <span class="bio-counter">{bio.length}/{BIO_MAX}</span>
    {/if}
  </div>
  {#snippet footer()}
    <div class="sheet-actions">
      <button
        class="sheet-submit"
        disabled={saving || !!problem}
        type="button"
        onclick={() => void save().then(() => (editing = null))}
      >
        {saving ? t('Saving…') : t('Save profile')}
      </button>
    </div>
  {/snippet}
</OverlaySheet>

<style>
  .status-line {
    min-height: 1.5em;
  }
</style>

<script lang="ts">
  import AvatarBadge from '#lib/components/icons/AvatarBadge.svelte';
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
    const file = (event.currentTarget as HTMLInputElement).files?.[0];
    if (!file) return;
    error = undefined;
    try {
      avatar = await toAvatar(file);
      status = `Picture resized to 256×256 (${Math.ceil(avatar.bytes.length / 1024)} KB). Save to keep it.`;
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
      status = 'Saved.';
    } catch (e) {
      error = explain(e);
    } finally {
      saving = false;
    }
  }
</script>

<section class="settings-section" id="settings-profile">
  <h2>Profile</h2>
  <div class="card">
    {#if !loaded}
      <p class="section-intro">Loading…</p>
    {:else}
      <div class="profile-row">
        <div class="avatar-wrap">
          <button
            class="avatar-picker"
            type="button"
            aria-label="Change profile photo"
            onclick={() => photoInput?.click()}
          >
            <AvatarBadge size="md" {name} {avatar} />
            <span class="avatar-hint">Change</span>
          </button>
          {#if avatar}
            <button
              class="avatar-remove"
              type="button"
              aria-label="Remove photo"
              onclick={() => (avatar = null)}>×</button
            >
          {/if}
        </div>
        <input
          bind:this={photoInput}
          accept="image/*"
          class="sr-only"
          type="file"
          onchange={pick}
        />
        <div class="profile-copy">
          <input
            bind:value={name}
            maxlength={HANDLE_MAX}
            aria-label="Username"
            placeholder="Username"
            autocomplete="name"
          />
          <input
            bind:value={homeArea}
            maxlength={HOME_AREA_MAX}
            aria-label="Home area"
            placeholder="Home area, e.g. Brisbane, QLD"
          />
          <textarea
            bind:value={bio}
            maxlength={BIO_MAX}
            rows="3"
            aria-label="About you"
            placeholder="About you"></textarea>
          <span class="bio-counter">{bio.length}/{BIO_MAX}</span>
        </div>
      </div>
      <!-- Reserved height: messages appear without pushing the page down. -->
      <p class:profile-image-error={!!error} class:bio-saved={!error} class="status-line">
        {error ?? status ?? problem ?? ''}
      </p>
      <div class="actions">
        <button class="button-primary" disabled={saving || !!problem} type="button" onclick={save}>
          {saving ? 'Saving…' : 'Save profile'}
        </button>
      </div>
    {/if}
  </div>
</section>

<style>
  .status-line {
    min-height: 1.5em;
  }
</style>

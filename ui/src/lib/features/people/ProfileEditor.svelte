<script lang="ts">
  import AvatarImage from '#lib/components/AvatarImage.svelte';
  import { BIO_MAX, HOME_AREA_MAX, NAME_MAX, type Avatar } from '#lib/domain/people.ts';
  import { explain } from '#lib/services/account.ts';
  import { getPerson, saveProfile } from '#lib/services/people.ts';
  import { connection } from '#lib/stores/connection.svelte.ts';

  import { toAvatar } from './avatar.ts';

  let name = $state('');
  let bio = $state('');
  let homeArea = $state('');
  let avatar = $state<Avatar | null>(null);
  let status = $state<string>();
  let error = $state<string>();
  let saving = $state(false);
  let loaded = $state(false);

  $effect(() => {
    const current = connection.current;
    if (current.status !== 'connected') return;
    void getPerson(current.agent).then((person) => {
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
      status = 'Saved.';
    } catch (e) {
      error = explain(e);
    } finally {
      saving = false;
    }
  }
</script>

<section class="panel stack editor">
  <h2>Your profile</h2>
  {#if !loaded}
    <p class="muted">Loading…</p>
  {:else}
    <div class="row">
      <AvatarImage {avatar} {name} size={72} />
      <label>
        Picture
        <input type="file" accept="image/*" onchange={pick} />
      </label>
      {#if avatar}<button onclick={() => (avatar = null)}>Remove</button>{/if}
    </div>
    <label>Name <input bind:value={name} maxlength={NAME_MAX} /></label>
    <label
      >Home area <input
        bind:value={homeArea}
        maxlength={HOME_AREA_MAX}
        placeholder="e.g. Brisbane, QLD"
      /></label
    >
    <label>About you <textarea bind:value={bio} maxlength={BIO_MAX} rows="4"></textarea></label>
    <p class="status" class:error={!!error}>
      {error ?? status ?? (name.trim() ? '' : 'Add your name to save.')}
    </p>
    <button class="primary" onclick={save} disabled={saving || !name.trim()}>Save profile</button>
  {/if}
</section>

<style>
  .editor {
    max-width: 640px;
  }
  /* Reserved height: messages appear without pushing the page down. */
  .status {
    min-height: 1.5em;
    margin: 0;
    color: var(--text-soft);
  }
</style>

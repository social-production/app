<script lang="ts">
  import { placeOnMap } from '#lib/components/map/placeOnMap.ts';
  import type { Snippet } from 'svelte';

  import AvatarBadge from '#lib/components/icons/AvatarBadge.svelte';
  import PhotoViewer from '#lib/components/overlay/PhotoViewer.svelte';
  import { displayName, type Person } from '#lib/domain/people.ts';

  /** The header of web's ProfilePage (classes styled by styles/profile.css). */
  interface Props {
    person: Person;
    /** Unused since the count row shows followers only (Profiles mockups §2). */
    devices?: number;
    actions?: Snippet;
    /** Beside the photo, at the right (your own profile's Edit profile, Profiles mockup §2). */
    aside?: Snippet;
    /** Leading items of the count row (web's follower / following buttons). */
    counts?: Snippet;
    /** The cover photo's URL (ADR 0107). */
    cover?: string | null;
    /** Your own profile: the cover and the photo get camera buttons. */
    onCoverFile?: ((file: File) => void) | undefined;
    onPhotoFile?: ((file: File) => void) | undefined;
  }

  let { person, actions, aside, counts, cover = null, onCoverFile, onPhotoFile }: Props = $props();

  const picked = (event: Event, then?: (file: File) => void): void => {
    const input = event.currentTarget as HTMLInputElement;
    const file = input.files?.[0];
    if (file) then?.(file);
    input.value = '';
  };

  // web's "View <name> profile photo": the picture opens full screen in the PhotoViewer.
  const avatar = $derived(person.profile?.avatar ?? null);
  let photoOpen = $state(false);
  const photoUrl = $derived(
    photoOpen && avatar
      ? URL.createObjectURL(new Blob([avatar.bytes.slice()], { type: avatar.mime }))
      : null,
  );
  $effect(() => () => {
    if (photoUrl) URL.revokeObjectURL(photoUrl);
  });

  import '#lib/components/styles/profile.css';
  const map = placeOnMap();
</script>

{#snippet camera(label: string, onFile: (file: File) => void, cls: string)}
  <label class="hero-cam {cls}" aria-label={label} title={label}>
    <svg aria-hidden="true" viewBox="0 0 24 24"
      ><rect x="3" y="5" width="18" height="14" rx="2" /><circle cx="9" cy="10" r="2" /><path
        d="m21 16-5-5-8 8"
      /></svg
    >
    <input hidden type="file" accept="image/*" onchange={(e) => picked(e, onFile)} />
  </label>
{/snippet}

<section class="hero-section" class:has-cover={!!cover}>
  <div class="profile-cover" style:background-image={cover ? `url(${cover})` : null}>
    {#if onCoverFile}{@render camera('Change cover photo', onCoverFile, 'cover-cam')}{/if}
  </div>
  <div class="hero-topline">
    <div class="hero-main">
      <div class="hero-identity">
        {#if avatar}
          <button
            class="avatar-open"
            type="button"
            aria-label={`View ${person.profile?.name || displayName(person)} profile photo`}
            onclick={() => (photoOpen = true)}
          >
            <AvatarBadge size="md" name={person.profile?.name ?? ''} {avatar} />
          </button>
        {:else}
          <AvatarBadge size="md" name={person.profile?.name ?? ''} avatar={null} />
        {/if}
        {#if onPhotoFile}{@render camera('Change profile photo', onPhotoFile, 'photo-cam')}{/if}
        {#if aside}<div class="hero-aside">{@render aside()}</div>{/if}
        <div class="hero-copy">
          <div class="name-row">
            <h1>{displayName(person)}</h1>
          </div>
          {#if person.profile?.bio}
            <p class="profile-bio">{person.profile.bio}</p>
          {/if}
          {#if person.profile?.home_area}
            {@const area = person.profile.home_area}
            <!-- A place button that opens the map there (owner, 2026-11-23). -->
            <a
              class="profile-area"
              href={`https://www.openstreetmap.org/search?query=${encodeURIComponent(area)}`}
              rel="noopener noreferrer"
              onclick={(e) => {
                if (!map || e.metaKey || e.ctrlKey || e.shiftKey) return;
                e.preventDefault();
                map.show(area);
              }}
            >
              <svg aria-hidden="true" viewBox="0 0 24 24"
                ><path d="M12 21s-7-6.2-7-11a7 7 0 0 1 14 0c0 4.8-7 11-7 11z" /><circle
                  cx="12"
                  cy="10"
                  r="2.5"
                /></svg
              >{area}
            </a>
          {/if}
          <!-- Followers and following only, as drawn; standing and vouches live in the ⋮ menu's Trust sheet (owner, 2026-11-24). -->
          <div class="count-row">
            {@render counts?.()}
          </div>
        </div>
      </div>
      {@render actions?.()}
    </div>
  </div>
</section>

<PhotoViewer
  url={photoUrl}
  alt={`${person.profile?.name || displayName(person)} profile photo`}
  onClose={() => (photoOpen = false)}
/>
